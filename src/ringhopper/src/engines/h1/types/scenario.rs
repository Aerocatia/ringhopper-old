use std::collections::HashMap;

use crate::engines::h1::definitions::*;
use crate::engines::h1::*;
use crate::error::*;
use crate::types::*;
use crate::engines::h1::TagReference;

use macros::*;
use macros::terminal::*;
use strings::*;

use riat::{Compiler, NodeData, NodeType, PrimitiveType, ValueType, CompiledNode, CompileError};

/// Trait for compiling scripts for scenario tags.
pub trait ScriptCompiler {
    /// Compile scripts in the scenario tag.
    fn compile_scripts<F>(&mut self, target: &EngineTarget, hud_message_text: &HUDMessageText, hud_globals: &HUDGlobals, resolve_object_fn: &mut F) -> ErrorMessageResult<Vec<CompileError>> where F: FnMut(&str) -> ErrorMessageResult<Option<TagGroup>>;
}

fn generate_script_node_id(index: Option<usize>) -> u32 {
    match index {
        None => u32::MAX,
        Some(n) if n > u16::MAX as usize => panic!("Tried to generate an ID with an index that exceeds a 16-bit signed integer. THIS IS A BUG!"),
        Some(n) => ((0x6373u16.overflowing_add(n as u16).0 as u32) << 16) | (n as u32) | 0x80000000
    }
}

fn handle_node<'a, F>(scenario: &Scenario,
                      hud_message_text: &HUDMessageText,
                      hud_globals: &HUDGlobals,
                      string_data: &mut Vec<u8>,
                      past_strings: &mut HashMap<&'a str, u32>,
                      index: usize,
                      new_nodes: &mut Vec<ScenarioScriptNode>,
                      compiled_nodes: &'a [CompiledNode],
                      references: &mut Vec<TagReference>,
                      resolve_object_fn: &mut F) -> ErrorMessageResult<()> where F: FnMut(&str) -> ErrorMessageResult<Option<TagGroup>> {
    let n = &compiled_nodes[index];

    // Determine the flags
    let mut flags = ScenarioScriptNodeFlags::default();
    flags.is_garbage_collectable = true;
    match n.get_type() {
        NodeType::Primitive(primitive_type) => {
            flags.is_primitive = true;
            match primitive_type {
                PrimitiveType::Static => (),
                PrimitiveType::Local => { flags.is_global = true; flags.is_local_variable = true; },
                PrimitiveType::Global => flags.is_global = true
            }
        },
        NodeType::FunctionCall(engine_function) => {
            flags.is_script_call = !engine_function;
        }
    };

    // Add our node
    new_nodes.push(ScenarioScriptNode {
        salt: (generate_script_node_id(Some(index)) >> 16) as u16,
        index_union: n.get_index().unwrap_or(n.get_value_type() as u16),
        _type: ScenarioScriptValueType::from_u16(n.get_value_type() as u16).unwrap(),
        flags,
        next_node: generate_script_node_id(n.get_next_node_index()),
        data: match n.get_data() {
            None => {
                macro_rules! find_in_array {
                    ($arr:expr, $arr_name:expr, $what:expr, $index_type:ty, $check_none:expr) => {{
                        let string_data = $what.to_ascii_lowercase();

                        if $check_none && string_data == "none" {
                            Ok(!0)
                        }
                        else {
                            let mut index = None;
                            for i in 0..$arr.len() {
                                if $arr[i].name.to_str().to_ascii_lowercase() == string_data {
                                    index = Some(i);
                                    break;
                                }
                            }
                            const MAX_INDEX: $index_type = <$index_type>::MAX;
                            match index {
                                Some(n) if n >= MAX_INDEX as usize => {
                                    Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_found_in_array_index_overflow"),
                                                                              string_data=string_data,
                                                                              arr_name=$arr_name,
                                                                              index=n,
                                                                              max_index=MAX_INDEX)))
                                },
                                Some(n) => Ok(n as $index_type),
                                None => {
                                    let msg = if string_data == "none" {
                                        format!(get_compiled_string!("engine.h1.types.scenario.error_compile_could_not_find_in_array_placeholder"), string_data=string_data, arr_name=$arr_name)
                                    }
                                    else {
                                        format!(get_compiled_string!("engine.h1.types.scenario.error_compile_could_not_find_in_array"), string_data=string_data, arr_name=$arr_name)
                                    };
                                    Err(ErrorMessage::AllocatedString(msg))
                                }
                            }
                        }
                    }};
                    ($arr:expr, $arr_name:expr, $what:expr, $index_type:ty) => {{
                        find_in_array!($arr, $arr_name, $what, $index_type, true)
                    }};
                    ($arr:expr, $arr_name:expr, $what:expr) => {{
                        find_in_array!($arr, $arr_name, $what, i16)
                    }};
                    ($arr:expr, $arr_name:expr) => {{
                        find_in_array!($arr, $arr_name, n.get_string_data().unwrap())
                    }}
                }

                macro_rules! find_in_array_and_make_value {
                    ($arr:expr, $arr_name:expr) => {{
                        ScenarioScriptNodeValue { short_int: find_in_array!($arr, $arr_name)? }
                    }}
                }

                macro_rules! find_tag {
                    ($group:expr) => {{
                        let tag_str = n.get_string_data().unwrap();
                        let mut found = false;
                        let reference = TagReference::from_path_and_group(tag_str, $group)?;
                        for i in references.into_iter() {
                            if reference.eq(i) {
                                found = true;
                                break;
                            }
                        }
                        if !found {
                            references.push(reference);
                        }
                        ScenarioScriptNodeValue::default()
                    }}
                }

                match n.get_value_type() {
                    ValueType::CutsceneRecording => find_in_array_and_make_value!(scenario.recorded_animations, "recorded animations"),
                    ValueType::AiCommandList => find_in_array_and_make_value!(scenario.command_lists, "command lists"),
                    ValueType::Conversation => find_in_array_and_make_value!(scenario.ai_conversations, "AI conversations"),
                    ValueType::DeviceGroup => find_in_array_and_make_value!(scenario.device_groups, "device groups"),
                    ValueType::TriggerVolume => find_in_array_and_make_value!(scenario.trigger_volumes, "trigger volumes"),
                    ValueType::CutsceneTitle => find_in_array_and_make_value!(scenario.cutscene_titles, "cutscene titles"),
                    ValueType::CutsceneFlag => find_in_array_and_make_value!(scenario.cutscene_flags, "cutscene flags"),
                    ValueType::CutsceneCameraPoint => find_in_array_and_make_value!(scenario.cutscene_camera_points, "cutscene cutscene_camera_points"),
                    ValueType::HudMessage => find_in_array_and_make_value!(hud_message_text.messages, "HUD message text's messages"),
                    ValueType::Navpoint => find_in_array_and_make_value!(hud_globals.waypoint_arrows, "HUD globals's waypoint arrows"),
                    ValueType::ObjectName => find_in_array_and_make_value!(scenario.object_names, "object names"),

                    /*
                     * Basically we have a 32-bit integer here which is actually a bitfield.
                     *
                     * 0xXX XX XXXX
                     *   ^  ^  ^
                     *   |  |  encounter
                     *   |  squad (or 00 if no squad)
                     *   80 if encounter/squad, 40 if encounter/platoon. else 00 if just encounter
                     *
                     * We take the encounter input, and we check if there is a "/".
                     *
                     * If so, we treat everything before the "/" as the encounter name and everything after as a subindex. Otherwise, the full string is the encounter name.
                     *
                     * Look for the encounter and set the lower 16 bits to the encounter index.
                     *
                     * Next, if we have a subindex, look for a squad with the same name as the subindex.
                     * - If we find a squad, we set the upper 8 bits to 0x80.
                     * - If not, look for a platoon, and if we find one, we set the upper 8 bits to 0x40.
                     * - Otherwise, error.
                     *
                     * Set the second highest 8 bits to the subindex index, or 0 if we have no subindex.
                     *
                     * TODO: Determine if subindex is signed.
                     */
                    ValueType::Ai => {
                        let tag_str = n.get_string_data().unwrap();
                        let (encounter_name, sub_index) = match tag_str.find('/') {
                            Some(n) => {
                                let (encounter, sub_index) = tag_str.split_at(n);
                                (encounter, Some(&sub_index[1..]))
                            },
                            None => (tag_str, None)
                        };

                        let encounter_index = find_in_array!(scenario.encounters, "encounters", encounter_name, i16, false)? as usize;

                        let (sub_index_type, sub_index) = match sub_index {
                            None => (0x00, 0x00),
                            Some(sub_index) => {
                                const MAX_INDEX: usize = i8::MAX as usize;
                                let squad = find_in_array!(scenario.encounters[encounter_index].squads, "squads", sub_index, usize, false);
                                let platoon = find_in_array!(scenario.encounters[encounter_index].platoons, "platoons", sub_index, usize, false);

                                match squad {
                                    Ok(n) if n > MAX_INDEX => {
                                        return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_found_squad_or_platoon_index_overflow"),
                                                                                         array_type="squads",
                                                                                         index=n,
                                                                                         max_index=MAX_INDEX,
                                                                                         sub_index=sub_index,
                                                                                         encounter=encounter_name)));
                                    },
                                    Ok(n) => (0x80, n),
                                    Err(_) => match platoon {
                                        Ok(n) if n > i8::MAX as usize => {
                                            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_found_squad_or_platoon_index_overflow"),
                                                                                              array_type="platoons",
                                                                                              index=n,
                                                                                              max_index=MAX_INDEX,
                                                                                              sub_index=sub_index,
                                                                                              encounter=encounter_name)));
                                        },
                                        Ok(n) => (0x40, n),
                                        Err(_) => {
                                            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_could_not_find_squad_or_platoon"), sub_index=sub_index, encounter=encounter_name)))
                                        }
                                    }
                                }
                            }
                        };

                        ScenarioScriptNodeValue { unsigned_long_int: ((sub_index_type as u32) << 24) |
                                                                     ((sub_index as u32) << 16) |
                                                                     (encounter_index as u32) }
                    }

                    ValueType::Sound => find_tag!(TagGroup::Sound),
                    ValueType::Effect => find_tag!(TagGroup::Effect),
                    ValueType::Damage => find_tag!(TagGroup::DamageEffect),
                    ValueType::LoopingSound => find_tag!(TagGroup::SoundLooping),
                    ValueType::AnimationGraph => find_tag!(TagGroup::ModelAnimations),
                    ValueType::ActorVariant => find_tag!(TagGroup::ActorVariant),
                    ValueType::DamageEffect => find_tag!(TagGroup::DamageEffect),
                    ValueType::ObjectDefinition => {
                        let tag_reference = n.get_string_data().unwrap();
                        let group = resolve_object_fn(n.get_string_data().unwrap())?
                                    .ok_or_else(|| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_could_not_find_object"), tag_reference=tag_reference)))?;
                        find_tag!(group)
                    },

                    _ => ScenarioScriptNodeValue::default()
                }
            },
            Some(data) => match data {
                NodeData::Boolean(b) => ScenarioScriptNodeValue { bool_int: b as i8 },
                NodeData::Short(s) => ScenarioScriptNodeValue { short_int: s },
                NodeData::Long(l) => ScenarioScriptNodeValue { long_int: l },
                NodeData::NodeOffset(o) => ScenarioScriptNodeValue { unsigned_long_int: generate_script_node_id(Some(o)) },
                NodeData::Real(f) => ScenarioScriptNodeValue { real: f }
            }
        },
        string_offset: match n.get_string_data() {
            Some(n) => {
                match past_strings.get(n) {
                    Some(n) => *n,
                    None => {
                        let offset = string_data.len() as u32;
                        past_strings.insert(n, offset);
                        string_data.reserve(n.len() + 1);
                        string_data.extend_from_slice(n.as_bytes());
                        string_data.push(0);
                        offset
                    }
                }
            }
            None => 0
        }
    });

    Ok(())
}

impl ScriptCompiler for Scenario {
    fn compile_scripts<F>(&mut self, target: &EngineTarget, hud_message_text: &HUDMessageText, hud_globals: &HUDGlobals, resolve_object_fn: &mut F) -> ErrorMessageResult<Vec<CompileError>> where F: FnMut(&str) -> ErrorMessageResult<Option<TagGroup>> {
        let mut compiler = Compiler::new(target.script_compile_target, riat::CompileEncoding::UTF8);

        // Load scripts
        for source in &mut self.source_files {
            // Workaround for c10 and d20 having non-ASCII bytes, making Rust's UTF8 parser fail since it is not UTF-8.
            //
            // We copy the source file and replace non-ASCII bytes with question marks.
            //
            // TODO: Remove this when we have a tag bludgeoner and don't clone the source data. That will let the script compiler just error if it is invalid.
            let mut source_copy = source.source.clone();
            let mut contains_non_ascii = false;
            for c in &mut source_copy {
                if !c.is_ascii() {
                    contains_non_ascii = true;
                    *c = '?' as u8;
                }
            }
            if contains_non_ascii {
                eprintln_warn!(get_compiled_string!("engine.h1.types.scenario.contains_non_ascii"), file=source.name.to_str());
            }

            compiler.read_script_data(&format!("{}.hsc", source.name.to_str()), &source_copy)
                    .map_err(|e| {
                        let file = e.get_file();
                        let message = e.get_message();
                        let (line,column) = e.get_position();
                        ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_failed_to_compile_scripts"), file=file, message=message, line=line, column=column))
                    })?;
        }

        // Compile scripts
        let script_data = compiler.digest_tokens()
                                  .map_err(|e| {
                                      let file = e.get_file();
                                      let message = e.get_message();
                                      let (line,column) = e.get_position();
                                      ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_failed_to_compile_scripts"), file=file, message=message, line=line, column=column))
                                  })?;

        // First our scripts
        let mut new_scripts = Vec::new();
        let new_scripts_c = script_data.get_scripts();
        new_scripts.reserve(new_scripts_c.len());
        for s in new_scripts_c {
            let mut parameters = Vec::new();
            let parameters_c = s.get_parameters();
            parameters.reserve(parameters_c.len());
            for p in parameters_c {
                parameters.push(ScenarioScriptParameter { name: String32::from_str(p.get_name())?, return_type: ScenarioScriptValueType::from_u16(p.get_value_type() as u16).unwrap() })
            }
            new_scripts.push(ScenarioScript {
                name: String32::from_str(s.get_name())?,
                script_type: ScenarioScriptType::from_u16(s.get_type() as u16).unwrap(),
                return_type: ScenarioScriptValueType::from_u16(s.get_value_type() as u16).unwrap(),
                root_expression_index: generate_script_node_id(Some(s.get_first_node_index())),
                parameters: Reflexive { blocks: parameters }
            });
        }

        // Next our globals
        let new_globals_c = script_data.get_globals();
        let mut new_globals = Vec::new();
        new_globals.reserve(new_globals_c.len());
        for g in new_globals_c {
            new_globals.push(ScenarioGlobal {
                name: String32::from_str(g.get_name())?,
                _type: ScenarioScriptValueType::from_u16(g.get_value_type() as u16).unwrap(),
                initialization_expression_index: generate_script_node_id(Some(g.get_first_node_index()))
            });
        }

        // Lastly our nodes
        let new_nodes_c = script_data.get_nodes();
        let node_count = new_nodes_c.len();
        let max_node_count = target.max_script_nodes;

        // Check if max node count exceeded
        if node_count > max_node_count {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_max_node_count_exceeded"), node_count=node_count, max_nodes=max_node_count)));
        }

        // Also require that there are 32 nodes free to allow the console to be used
        if node_count + 32 > max_node_count {
            return Err(ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_max_node_count_exceeded_console"), node_count=node_count, max_nodes=max_node_count)));
        }

        // Make new nodes here
        let mut new_nodes = Vec::new();
        new_nodes.reserve(node_count);
        let mut string_data = Vec::new();
        let mut past_strings = HashMap::<&str, u32>::new();

        // We also want references here
        let mut references = Vec::<TagReference>::new();

        for i in 0..node_count {
            let result = handle_node(self, hud_message_text, hud_globals, &mut string_data, &mut past_strings, i, &mut new_nodes, new_nodes_c, &mut references, resolve_object_fn);
            result.map_err(|e| ErrorMessage::AllocatedString(format!(get_compiled_string!("engine.h1.types.scenario.error_compile_failed_to_compile_scripts"),
                                                                     file=self.source_files[new_nodes_c[i].get_file()].name.to_str().to_owned() + ".hsc",
                                                                     line=new_nodes_c[i].get_line(),
                                                                     column=new_nodes_c[i].get_column(),
                                                                     message=e)))?;
        }

        string_data.resize(string_data.len() + 1024, 0); // needed to ensure the console still works since the console dynamically modifies the script data

        // Now make our script syntax data!
        let mut syntax_data = Vec::new();
        let node_tag_size = ScenarioScriptNode::tag_size();
        let table_header = ScenarioScriptNodeTable {
            name: String32::from_str("script node").unwrap(),
            maximum_count: max_node_count as u16,
            one: 1,
            size: node_count as u16,
            element_size: node_tag_size as u16,
            data: 0x64407440, // d@t@ fourcc,
            count: node_count as u16,
            next_id: (generate_script_node_id(Some(node_count as usize)) >> 16) as u16,
            first_element_ptr: 0
        };

        // Write the nodes
        let mut node_offset = tag_size_instance(&table_header);
        let expected_length = node_offset + max_node_count * node_tag_size;
        syntax_data.resize(expected_length, 0);
        table_header.into_tag(&mut syntax_data, 0, node_offset).unwrap();
        for n in new_nodes {
            let node_offset_end = node_offset + node_tag_size;
            n.into_tag(&mut syntax_data, node_offset, node_offset_end).unwrap();
            node_offset = node_offset_end;
        }

        // Make our new references array
        let mut reference_array = Vec::new();
        reference_array.reserve(references.len());
        for r in references {
            reference_array.push( ScenarioReference { reference: r });
        }

        self.scripts = Reflexive { blocks: new_scripts };
        self.globals = Reflexive { blocks: new_globals };
        self.references = Reflexive { blocks: reference_array };
        self.script_string_data = string_data;
        self.script_syntax_data = syntax_data;

        Ok(script_data.get_warnings().to_owned())
    }
}
