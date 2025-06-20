use colored::Colorize;
use term_table::{Table, TableStyle, row::Row, table_cell::TableCell};

pub trait NamedEntity {
  fn id(&self) -> u64;
  fn name(&self) -> &str;
}

pub fn output_named_entities_raw<T: NamedEntity>(values: &[T]) {
  for entity in values {
    println!("\"{}\"", entity.name());
  }
}

pub fn output_named_entities_table<T: NamedEntity>(values: &[T], title: &str) {
  let mut table = Table::new();
  table.style = TableStyle::thin();

  let header = Row::new(vec![
    TableCell::new("ID".bold().white()),
    TableCell::new(title.bold().white()),
  ]);

  table.add_row(header);

  for entity in values {
    let row = Row::new(vec![
      TableCell::new(entity.id()),
      TableCell::new(entity.name()),
    ]);

    table.add_row(row);
  }

  println!("{}", table.render());
}
