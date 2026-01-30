// Code smells for Rex to find:
// - unwrap without error handling
// - unused variables
// - inefficient string concatenation

fn main() {
    let unused_var = 42;
    
    let result = std::fs::read_to_string("config.json").unwrap();
    
    let mut output = String::new();
    for i in 0..100 {
        output = output + &i.to_string() + ", ";
    }
    
    println!("{}", output);
}

fn uncalled_function() {
    let x = 5;
    let y = 10;
    // TODO: implement this
}
