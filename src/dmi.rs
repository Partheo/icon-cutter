mod chunk;
mod crc;
mod error;

use error::DmiError;
use std::convert::TryFrom;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;

/// The PNG magic header
const MAGIC: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Dmi {
	pub header: [u8; 8],
	pub chunk_ihdr: chunk::Chunk,
	pub chunk_ztxt: chunk::Chunk,
	pub chunk_idat: chunk::Chunk,
	pub chunk_iend: chunk::Chunk,
	pub other_chunks: Vec<chunk::Chunk>,
}

impl Dmi {
	pub fn write_ztxt_chunk(&mut self, new_text: String) -> Result<bool, error::DmiError> {
		let return_val = self.chunk_ztxt.write_ztxt_chunk(new_text).unwrap();
		Ok(return_val)
	}

	pub fn save(&mut self, path: String) -> Result<bool, error::DmiError> {
		let mut dmi_bytes: Vec<u8> = vec![];

		dmi_bytes.extend_from_slice(&self.header);
		dmi_bytes = self.chunk_ihdr.write_to_vec(dmi_bytes).unwrap();
		/* //Let's drop the other chunks for now. None of them are necessary for dmi files.
		for current_chunk in &mut self.other_chunks {
			dmi_bytes = current_chunk.write_to_vec(dmi_bytes).unwrap();
		}
		*/
		dmi_bytes = self.chunk_ztxt.write_to_vec(dmi_bytes).unwrap();
		dmi_bytes = self.chunk_idat.write_to_vec(dmi_bytes).unwrap();
		dmi_bytes = self.chunk_iend.write_to_vec(dmi_bytes).unwrap();

		let path = Path::new(&path);
		let mut file = File::create(&path).expect("Failed create dmi path");
		file.write_all(&dmi_bytes).expect("Failed to write dmi");

		Ok(true)
	}
}

pub fn dmi_from_vec(bytes_vec: &[u8]) -> Result<Dmi, error::DmiError> {
	let header = <[u8; 8]>::try_from(&bytes_vec[0..8]).unwrap();

	if header != MAGIC {
		return Err(DmiError::MagicMismatch(header));
	}

	let mut index = 8; //Without the magic header.

	let mut chunk_ihdr = chunk::Chunk::default();
	let mut chunk_ztxt = chunk::Chunk::default();
	let mut chunk_idat = chunk::Chunk::default();
	let chunk_iend;
	let mut other_chunks: Vec<chunk::Chunk> = vec![];

	loop {
		let (current_chunk, new_index) = chunk::Chunk::read_from_vec(&bytes_vec, index)
			.expect("Error while reading the file's chunks");
		index = new_index;
		match &current_chunk.chunk_type {
			b"IHDR" => chunk_ihdr = current_chunk.clone(),
			b"zTXt" => chunk_ztxt = current_chunk.clone(),
			b"IDAT" => chunk_idat = current_chunk.clone(),
			b"IEND" => {
				chunk_iend = current_chunk.clone();
				break;
			}
			_ => other_chunks.push(current_chunk.clone()),
		}
	}

	let dmi_result = Dmi {
		header,
		chunk_ihdr,
		chunk_ztxt,
		chunk_idat,
		chunk_iend,
		other_chunks,
	};
	Ok(dmi_result)
}
