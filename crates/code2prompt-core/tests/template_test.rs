use code2prompt_core::template::{extract_undefined_variables, handlebars_setup, render_template};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_handlebars_setup() {
        let template_str = "Hello, {{name}}!";
        let template_name = "test_template";

        // Call the handlebars_setup function
        let handlebars =
            handlebars_setup(template_str, template_name).expect("Failed to set up Handlebars");

        // Prepare the data
        let data = json!({
            "name": "Bernard"
        });

        // Render the template
        let rendered = render_template(&handlebars, "test_template", &data);

        // Assert the result
        match rendered {
            Ok(output) => assert_eq!(output, "Hello, Bernard!"),
            Err(e) => panic!("Template rendering failed: {}", e),
        }
    }

    #[test]
    fn test_extract_undefined_variables() {
        let template_str = "{{name}} is learning {{language}} and {{framework}}!";
        let variables = extract_undefined_variables(template_str);
        assert_eq!(variables, vec!["name", "language", "framework"]);
    }

    #[test]
    fn test_render_template() {
        let template_str = "{{greeting}}, {{name}}!";
        let template_name = "test_template";
        let handlebars = handlebars_setup(template_str, template_name).unwrap();
        let data = json!({ "greeting": "Hello", "name": "Bernard" });
        let rendered = render_template(&handlebars, template_name, &data);

        match rendered {
            Ok(output) => assert_eq!(output, "Hello, Bernard!"),
            Err(e) => panic!("Template rendering failed: {}", e),
        }
    }

    #[test]
    fn test_xml_template_contains_backtick_fences() {
        use code2prompt_core::template::{handlebars_setup, render_template};
        let template_str = include_str!("../src/default_template_xml.hbs");
        let handlebars = handlebars_setup(template_str, "xml").unwrap();
        let data = serde_json::json!({
            "absolute_code_path": "/some/path",
            "source_tree": "tree",
            "files": [{"path": "main.rs", "extension": "rs", "code": "fn main() {}"}],
            "no_codeblock": false
        });
        let rendered = render_template(&handlebars, "xml", &data).unwrap();
        assert!(rendered.contains("```rs"));
        assert!(rendered.contains("fn main() {}"));
    }

    #[test]
    fn test_xml_template_no_fences_when_no_codeblock() {
        use code2prompt_core::template::{handlebars_setup, render_template};
        let template_str = include_str!("../src/default_template_xml.hbs");
        let handlebars = handlebars_setup(template_str, "xml").unwrap();
        let data = serde_json::json!({
            "absolute_code_path": "/some/path",
            "source_tree": "tree",
            "files": [{"path": "main.rs", "extension": "rs", "code": "fn main() {}"}],
            "no_codeblock": true
        });
        let rendered = render_template(&handlebars, "xml", &data).unwrap();
        assert!(!rendered.contains("```"));
        assert!(rendered.contains("fn main() {}"));
    }
}
