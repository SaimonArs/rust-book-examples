pub struct Post {
    content: String,
}

pub struct DraftPost {
    content: String,
}

impl Post {
    pub fn new() -> DraftPost {
        DraftPost {
            content: String::new(),
        }
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl DraftPost {
    pub fn add_text(&mut self, text: &str) {
        self.content.push_str(text);
    }

    pub fn request_review(self) -> PendingReviewPostV1 {
        PendingReviewPostV1 {
            content: self.content,
        }
    }
}

pub struct PendingReviewPostV1 {
    content: String,
}

impl PendingReviewPostV1 {
    pub fn approve(self) -> PendingReviewPostV2 {
        PendingReviewPostV2 {content: self.content}
    }

    pub fn reject(self) -> DraftPost {
        DraftPost {
            content: self.content,
        }
    }
}

pub struct PendingReviewPostV2 {
    content: String,
}

impl PendingReviewPostV2 {
    pub fn approve(self) -> Post {
        Post { content: self.content }
    }

    pub fn reject(self) -> DraftPost {
        DraftPost {
            content: self.content,
        }
    }
}