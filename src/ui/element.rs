use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Element {
    pub tag: String,
    pub props: HashMap<String, Value>,
    pub children: Vec<Element>,
    pub text: Option<String>,
}

impl Element {
    pub fn new(tag: &str) -> Self {
        Element {
            tag: tag.to_string(),
            props: HashMap::new(),
            children: Vec::new(),
            text: None,
        }
    }

    pub fn text(content: &str) -> Self {
        Element {
            tag: "text".to_string(),
            props: HashMap::new(),
            children: Vec::new(),
            text: Some(content.to_string()),
        }
    }

    pub fn prop<T: Into<Value>>(mut self, key: &str, value: T) -> Self {
        self.props.insert(key.to_string(), value.into());
        self
    }

    pub fn class(self, class: &str) -> Self {
        self.prop("class", class)
    }

    pub fn id(self, id: &str) -> Self {
        self.prop("id", id)
    }

    pub fn child(mut self, child: Element) -> Self {
        self.children.push(child);
        self
    }

    pub fn children(mut self, children: Vec<Element>) -> Self {
        self.children.extend(children);
        self
    }
}

// Helper functions for common elements
pub fn div() -> Element {
    Element::new("div")
}

pub fn span() -> Element {
    Element::new("span")
}

pub fn h1() -> Element {
    Element::new("h1")
}

pub fn h2() -> Element {
    Element::new("h2")
}

pub fn h3() -> Element {
    Element::new("h3")
}

pub fn p() -> Element {
    Element::new("p")
}

pub fn button() -> Element {
    Element::new("button")
}

pub fn input() -> Element {
    Element::new("input")
}

pub fn form() -> Element {
    Element::new("form")
}

pub fn nav() -> Element {
    Element::new("nav")
}

pub fn header() -> Element {
    Element::new("header")
}

pub fn main() -> Element {
    Element::new("main")
}

pub fn footer() -> Element {
    Element::new("footer")
}

pub fn section() -> Element {
    Element::new("section")
}

pub fn article() -> Element {
    Element::new("article")
}

pub fn ul() -> Element {
    Element::new("ul")
}

pub fn li() -> Element {
    Element::new("li")
}

pub fn a() -> Element {
    Element::new("a")
}

pub fn img() -> Element {
    Element::new("img")
}

pub fn text(content: &str) -> Element {
    Element::text(content)
}

// Added label helper function
pub fn label() -> Element {
    Element::new("label")
}
