use crate::database::Database;

pub struct Indexer<'a> {
	database: &'a Database,
}

impl<'a> Indexer<'a> {
	pub fn new(database: &Database) -> Indexer {
		Indexer { database: database }
	}

	pub async fn insert_from_csv(csv_path: String) {}
}

use std::fs::File;
use std::io::{BufRead, BufReader, Cursor, Write, Seek, SeekFrom};
use std::collections::HashMap;
use regex::Regex;

pub struct IndexableCSV {
	path: String,
	file: File,
	header: Vec<String>,
	offsets: Vec<usize>,
}

impl IndexableCSV {
	pub fn new(path: String) -> IndexableCSV {
		let mut file = File::open(&path).expect("Something went wrong reading the file");

		let header = IndexableCSV::get_file_header(&mut file);
		let offsets = IndexableCSV::get_file_offsets(&mut file);

		IndexableCSV {
			path,
			file,
			header,
			offsets,
		}
	}

	pub fn get_file_header(file: &mut File) -> Vec<String> {
		let mut buf: String = String::from("");

		BufReader::new(file).read_line(&mut buf);

		buf.split(',')
			.map(|x| String::from(x))
			.collect::<Vec<String>>()
	}

	pub fn get_file_offsets(file: &mut File) -> Vec<usize> {
		BufReader::new(file)
			.lines()
			.skip(1)
			.map(|line| {
				line
					.unwrap_or(String::from(""))
					.len()
			})
			.collect::<Vec<usize>>()
	}

	pub fn get_row(&mut self, i: usize) -> HashMap<&String, String> {
		self.file.seek(SeekFrom::Start(self.offsets[i].try_into().unwrap()));

		let mut buf: String = String::from("");
		BufReader::new(&mut self.file).read_line(&mut buf);

		let row = Regex::new(r"").unwrap()
			.split(&buf)
			.map(|x| x.to_string())
			.collect::<Vec<String>>();

		let mut dict = HashMap::new();
		for (ind, column) in self.header.iter().enumerate() {
			dict.insert(column, row[ind].clone());
		}

		dict
	}

	pub fn len(&self) -> usize {
		self.offsets.len()
	}
}
