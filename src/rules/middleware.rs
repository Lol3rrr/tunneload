use crate::http::Request;
use crate::rules::Action;

#[derive(Clone, Debug, PartialEq)]
pub enum Direction {
    Request,
    Response,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Middleware {
    name: String,
    action: Action,
    apply_way: Direction,
}

impl Middleware {
    pub fn new(name: &str, action: Action) -> Self {
        let apply_way = match action {
            Action::RemovePrefix(_) => Direction::Request,
            Action::AddHeader(_, _) => Direction::Response,
        };

        Self {
            name: name.to_owned(),
            action,
            apply_way,
        }
    }

    pub fn apply(&self, req: &mut Request, direction: &Direction) {
        if *direction != self.apply_way {
            return;
        }

        self.action.apply(req)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_direction(&self) -> &Direction {
        &self.apply_way
    }
}

#[test]
fn new_remove_prefix() {
    let middleware = Middleware::new("Test", Action::RemovePrefix("/api/".to_owned()));

    assert_eq!(Direction::Request, *middleware.get_direction());
}

#[test]
fn new_add_header() {
    let middleware = Middleware::new(
        "Test",
        Action::AddHeader("test-key".to_owned(), "test-value".to_owned()),
    );

    assert_eq!(Direction::Response, *middleware.get_direction());
}
