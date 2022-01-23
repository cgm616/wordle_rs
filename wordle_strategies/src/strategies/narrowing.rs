// use wordle_rs::Strategy;

/// A hard- or easymode Wordle strategy that guesses the word that most narrows
/// the remaining search space.
///
/// The strategy can comply with hardmode rules, which mean that it will guess
/// the best word that incorporates all of the information it has gained.
/// On easymode, the strategy is able to guess the most-narrowing possible next
/// word. When it has only one guess left, the strategy will make sure to
/// always incorporate all of the possible information.
pub struct Narrowing {
    hardmode: bool,
}

// impl Strategy for Narrowing
