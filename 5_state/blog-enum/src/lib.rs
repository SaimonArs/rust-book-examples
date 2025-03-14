#[derive(PartialEq)]
enum State {
    Draft,
    PendingReview,
    Published
}

pub struct Post {
    state: State,
    content: String,
    approve_counter: u8,
}

impl Post {
    pub fn new() -> Self {
        Post {
            state: State::Draft,
            content: String::new(),
            approve_counter: 0,
        }
    }

    pub fn add_text(&mut self, text: &str) {
        if self.state == State::Draft {
            self.content.push_str(text)
        }
    }

    pub fn content(&self) -> &str {
        match self.state {
            State::Published => &self.content,
            _ => ""
        }
    }

    pub fn request_review(&mut self) {
        if self.state == State::Draft {
            self.state = State::PendingReview;
        }
    }

    pub fn reject(&mut self) {
        if self.state == State::PendingReview {
            self.approve_counter = 0;
            self.state = State::Draft;
        }
    }

    pub fn approve(&mut self) {
        if self.state == State::PendingReview {
            self.approve_counter += 1;
            if self.approve_counter == 2 {
                self.state = State::Published;
            }
        }
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