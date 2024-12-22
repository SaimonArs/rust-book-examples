fn main() {
    let a = [
    "Twelve drummers drumming",
    "Eleven pipers piping",
    "Ten lords a-leaping",
    "Nine ladies dancing",
    "Eight maids a-milking",
    "Seven swans a-swimming",
    "Six geese a-laying",
    "Five golden rings",
    "Four calling birds",
    "Three french hens",
    "Two turtle doves and"
    ];

    let mut b = 12;
    for _ in 0..12 {
        println!("On the twelfth day of Christmas, my true love sent to me");
        for i in b-1..11 {
            println!("{}", a[i]);
        }
        println!("A partridge in a pear tree");
        println!("{b}");
        b -= 1;
    }
}
