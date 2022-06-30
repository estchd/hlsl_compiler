mod compile;
mod compile_flags;
mod effect_compile_flags;

pub use compile::{
	compile_from_file,
	compile_from_file_to_file,
	CompileFromFileToFileError,
	ShaderMacro
};
pub use compile_flags::{
	CompileFlags
};
pub use effect_compile_flags::{
	EffectCompileFlags
};

#[cfg(test)]
mod test {
	use crate::{compile_from_file, compile_from_file_to_file};
	use crate::compile::OptionalInclude;
	use crate::CompileFlags;
	use crate::EffectCompileFlags;

	#[test]
	fn test_compile_from_file() {
		let (result, messages) = compile_from_file(
			"test_files/pixel_shader.hlsl".to_string(),
			None,
			OptionalInclude::None,
			"PShader".to_string(),
			"ps_5_0".to_string(),
			CompileFlags::empty(),
			EffectCompileFlags::empty()
		);

		if let Err(error) = result {
			if let Some(messages) = messages {
				eprintln!("Error messages:");
				print_error_messages(messages);
			}
			panic!("Compilation Error: {}", error);
		}
		if let Some(messages) = messages {
			eprintln!("Error messages:");
			print_error_messages(messages);
			panic!("Success but error messages")
		}
	}

	#[test]
	fn test_compile_from_file_to_file() {
		let (result, messages) = compile_from_file_to_file(
			"test_files/pixel_shader.hlsl".to_string(),
			"test_output/pixel_shader.cso".to_string(),
			None,
			OptionalInclude::None,
			"PShader".to_string(),
			"ps_5_0".to_string(),
			CompileFlags::empty(),
			EffectCompileFlags::empty()
		);

		if let Err(error) = result {
			if let Some(messages) = messages {
				eprintln!("Error messages:");
				print_error_messages(messages);
			}
			panic!("Compilation Error: {:?}", error);
		}
		if let Some(messages) = messages {
			eprintln!("Error messages:");
			print_error_messages(messages);
			panic!("Success but error messages")
		}
	}

	fn print_error_messages(message_blob: Vec<u8>) {
		let message_string = String::from_utf8(message_blob).unwrap();
		let messages = message_string.lines();
		for line in messages {
			eprintln!("{}", line);
		}
	}
}