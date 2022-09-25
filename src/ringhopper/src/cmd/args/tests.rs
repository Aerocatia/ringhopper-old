use super::ArgumentConstraints;
use super::ParsedArguments;
use super::Argument;

#[test]
fn argument_parsing() {
    let test_arguments = [
        Argument { long: "an-arg", short: 'a', description: "", parameter: Some("something"), multiple: false },
        Argument { long: "boring-arg", short: 'b', description: "", parameter: None, multiple: false },
        Argument { long: "cool-arg", short: 'c', description: "", parameter: Some("something"), multiple: true }
    ];

    // All arguments (short)
    let result = ParsedArguments::parse_arguments(&["-abc", "some arg", "another arg"], &test_arguments, &[], "", "", ArgumentConstraints::new()).unwrap();
    assert_eq!("some arg", result.named.get("an-arg").unwrap()[0]);
    assert!(result.named.get("boring-arg").unwrap().is_empty());
    assert_eq!("another arg", result.named.get("cool-arg").unwrap()[0]);

    // All arguments (long)
    let result = ParsedArguments::parse_arguments(&["--an-arg", "some arg", "--boring-arg", "--cool-arg", "another arg"], &test_arguments, &[], "", "", ArgumentConstraints::new()).unwrap();
    assert_eq!("some arg", result.named.get("an-arg").unwrap()[0]);
    assert!(result.named.get("boring-arg").unwrap().is_empty());
    assert_eq!("another arg", result.named.get("cool-arg").unwrap()[0]);

    // If we omit them
    let result = ParsedArguments::parse_arguments(&[], &test_arguments, &[], "", "", ArgumentConstraints::new()).unwrap();
    assert!(result.named.get("an-arg").is_none());
    assert!(result.named.get("boring-arg").is_none());
    assert!(result.named.get("cool-arg").is_none());

    // Some arguments can be used more than once.
    let result = ParsedArguments::parse_arguments(&["-ccc", "some arg", "another arg", "one more arg"], &test_arguments, &[], "", "", ArgumentConstraints::new()).unwrap();
    assert!(result.named.get("an-arg").is_none());
    assert!(result.named.get("boring-arg").is_none());
    assert_eq!(&["some arg", "another arg", "one more arg"], &result.named.get("cool-arg").unwrap()[..]);

    // If we forget the parameter for one and it doesn't work
    assert!(ParsedArguments::parse_arguments(&["--an-arg", "some arg", "--boring-arg", "--cool-arg"], &test_arguments, &[], "", "", ArgumentConstraints::new()).is_err());
    assert!(ParsedArguments::parse_arguments(&["-abc", "some arg"], &test_arguments, &[], "", "", ArgumentConstraints::new()).is_err());

    // If we forget the parameter for one and it works (be careful!)
    let result = ParsedArguments::parse_arguments(&["--an-arg", "--boring-arg", "--cool-arg", "another arg"], &test_arguments, &[], "", "", ArgumentConstraints::new()).unwrap();
    assert_eq!("--boring-arg", result.named.get("an-arg").unwrap()[0]);
    assert!(result.named.get("boring-arg").is_none());
    assert_eq!("another arg", result.named.get("cool-arg").unwrap()[0]);

    // Extra arguments?
    let result = ParsedArguments::parse_arguments(&["hello"], &test_arguments, &["extra"], "", "", ArgumentConstraints::new()).unwrap();
    assert_eq!("hello", result.extra[0]);

    // Extra arguments cannot start with hyphens.
    assert!(ParsedArguments::parse_arguments(&["-"], &test_arguments, &["extra"], "", "", ArgumentConstraints::new()).is_err());
    assert!(ParsedArguments::parse_arguments(&["--"], &test_arguments, &["extra"], "", "", ArgumentConstraints::new()).is_err());
    assert!(ParsedArguments::parse_arguments(&["-a"], &test_arguments, &["extra"], "", "", ArgumentConstraints::new()).is_err());
    assert!(ParsedArguments::parse_arguments(&["--a"], &test_arguments, &["extra"], "", "", ArgumentConstraints::new()).is_err());

    // Extra arguments do not care about position.
    let result = ParsedArguments::parse_arguments(&["--an-arg", "some arg", "hello", "--boring-arg", "--cool-arg", "another arg"], &test_arguments, &["extra"], "", "", ArgumentConstraints::new()).unwrap();
    assert_eq!("hello", result.extra[0]);
    assert_eq!("some arg", result.named.get("an-arg").unwrap()[0]);
    assert!(result.named.get("boring-arg").unwrap().is_empty());
    assert_eq!("another arg", result.named.get("cool-arg").unwrap()[0]);
}
