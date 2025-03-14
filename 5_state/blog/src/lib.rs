pub struct Post {
    state: Option<Box<dyn State>>,
    content: String,
}

impl Post {
    pub fn new() -> Self {
        Post {
            state: Some(Box::new(Draft {})),
            content: String::new(),
        }
    }

    pub fn add_text(&mut self, text: &str) {
        if self.state.as_ref().unwrap().is_editable() {
            self.content.push_str(text);
        }
    }

    pub fn content(&self) -> &str {
        self.state.as_ref().unwrap().content(self)
    }

    pub fn request_review(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.request_review())
        }
    }
    pub fn reject(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.reject())
        }
    }

    pub fn approve(&mut self) {
        if let Some(s) = self.state.take() {
            self.state = Some(s.approve())
        }
    }
}

trait State {
    fn request_review(self: Box<Self>) -> Box<dyn State>;
    fn approve(self: Box<Self>) -> Box<dyn State>;
    fn reject(self: Box<Self>) -> Box<dyn State>;
    fn is_editable(&self) -> bool {
        false
    }
    fn content<'a>(&self, _post: &'a Post) -> &'a str {
        ""
    }
    
}

struct Draft {}


impl State for Draft {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        Box::new(PendingReview {approve_counter: 0})
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn is_editable(&self) -> bool {
        true
    }
}

struct PendingReview {
    approve_counter: u8
}

impl State for PendingReview {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(mut self: Box<Self>) -> Box<dyn State> {
        self.approve_counter += 1;
        match self.approve_counter {
            2 => Box::new(Published {}),
            _ => self
        }
     
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        Box::new(Draft {})
    }
}

struct Published {}

impl State for Published {
    fn request_review(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn approve(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn reject(self: Box<Self>) -> Box<dyn State> {
        self
    }
    fn content<'a>(&self, post: &'a Post) -> &'a str {
        &post.content
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn add_text() {
        let mut post = Post::new();
        
        post.add_text("Test");

        assert_eq!("", post.content());

        post.request_review();

        post.approve();
        assert_eq!("", post.content());

        post.approve();
        
        assert_eq!("Test", post.content());
    }

    #[test]
    fn reject_pending_review() {
        let mut post = Post::new();

        post.add_text("Test");

        post.request_review();

        post.approve();

        assert_eq!("", post.content());

        post.reject();

        post.request_review();

        post.approve();

        post.approve();
        assert_eq!("Test", post.content());
    }

    #[test]
    fn editable() {
        let mut post = Post::new();

        post.add_text("Test");
        post.add_text("Test");

        post.request_review();

        post.add_text("Test");

        post.approve();

        post.add_text("Test");

        post.approve();

        post.add_text("Test");

        assert_eq!("TestTest", post.content());
    }

}