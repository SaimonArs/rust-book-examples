use blog_types::Post;

fn main() {
    let mut post = Post::new();

    post.add_text("I ate a salad for lunch today");

    let post = post.request_review();

    let post = post.approve();

    let mut post = post.reject();
    post.add_text("Test");

    let post = post.request_review();

    let post = post.approve();

    let post = post.approve();
    
    assert_eq!("I ate a salad for lunch todayTest", post.content());
}
