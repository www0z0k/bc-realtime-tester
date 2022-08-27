#![allow(non_snake_case)]
use crate::*;

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize, Clone, PartialEq)]
pub struct Char {
    pub x: i32,
    pub y: i32,
    pub account: String,
}

impl Char {
    pub fn new(name: String, x: i32, y: i32) -> Char {
        let char = Char {
            account: name, x: x, y: y,
        };
        char
    }
}