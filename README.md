# SpreadDnDsheet

## Description

SpreadDnD Sheet is a spreadsheet engine designed for Dungeons and Dragons character sheets. D&D character sheets are basically just spreadsheets, you have a bunch of stats which all depend on each other in some way. However, D&D character features are often expressed as how they impact character stats and abilties, and the open-ended nature of the game means that a it is impossible to know in advance all the things that may affect some piece of data.

The key feature of SpreadDnD Sheet is that the formula language also allows for cell formulas to affect other cells in a structured way to allow for easier translation from game rules to the formula language.

## Usage

There is currently a simple front end written in the rust GUI framework [iced](https://iced.rs/). You can run it with `cargo run --bin gui`.