use blog_enum::Post;

fn main() {
    let mut post = Post::new();

    post.add_text("Test, test");

    assert_eq!("", post.content());

    post.request_review();
    assert_eq!("", post.content());
    
    post.approve();
    assert_eq!("", post.content());

    post.reject();

    post.request_review();
    assert_eq!("", post.content());

    post.approve();
    assert_eq!("", post.content());

    post.approve();
    assert_eq!("Test, test", post.content());

    post.add_text("Test, test");
    assert_eq!("Test, test", post.content());
}
