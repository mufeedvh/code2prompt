use code2prompt::template::{extract_undefined_variables, handlebars_setup, render_template};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_handlebars_setup() {
        let template_str = "{{name}} is learning Rust!";
        let template_name = "test_template";
        let handlebars = handlebars_setup(template_str, template_name).unwrap();
        let data = json!({ "name": "Bernard" });
        let rendered = handlebars.render(template_name, &data).unwrap();
        assert_eq!(rendered, "Bernard is learning Rust!");
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
        assert_eq!(rendered, "Hello, Bernard!");
    }
}
