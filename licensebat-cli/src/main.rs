fn main() {
    println!("Hello, Licensebat!");
    // with the cli we're not constrained to normal files
    // we can also get into the dependency directory itself and
    // traverse it.
    // This basically means we can either parse a depdendency manifest
    // or some directory and that the collectors don't need a specific api. So, for each supported language a different strategy may be used.

    // The user has to provide a list of languages supported and then
    // for each language, a strategy will be used.
}
