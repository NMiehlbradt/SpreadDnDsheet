use dnd_spreadsheet::reactive;
use dnd_spreadsheet::language;

fn main() {
    println!("Hello, world!");

    let mut sheet: reactive::sheet::Sheet<language::ast::AST> = reactive::sheet::Sheet::new();

    let cell1 = sheet.add_cell("A1".to_string(), "5").unwrap();
    let cell2 = sheet.add_cell("A2".to_string(), "-A1 - -3").unwrap();
    let cell3 = sheet.add_cell("A3".to_string(), "{x: A1, y: A2}").unwrap();

    println!("A1: {:?}", sheet.get_cell_value(&cell1).unwrap());
    println!("A2: {:?}", sheet.get_cell_value(&cell2).unwrap());
    println!("A3: {:?}", sheet.get_cell_value(&cell3).unwrap());

    sheet.update_cell(&cell1, "-2");

    println!("A1: {:?}", sheet.get_cell_value(&cell1).unwrap());
    println!("A2: {:?}", sheet.get_cell_value(&cell2).unwrap());
    println!("A3: {:?}", sheet.get_cell_value(&cell3).unwrap());

    println!("A1: {}", sheet.get_ast_s_expr(&cell1));
    println!("A2: {}", sheet.get_ast_s_expr(&cell2));
    println!("A3: {}", sheet.get_ast_s_expr(&cell3));
}
