use std::ffi::{CString, NulError};
use std::fs::File;
use std::io::Write;
use std::mem::MaybeUninit;
use std::ptr::{copy, null};
use windows::Win32::Graphics::Direct3D::{D3D_SHADER_MACRO, ID3DBlob, ID3DInclude};
use crate::compile_flags::CompileFlags;
use crate::effect_compile_flags::EffectCompileFlags;
use thiserror::Error;
use widestring::error::ContainsNul;
use widestring::U16CString;
use windows::core::{IntoParam, PCSTR, PCWSTR};
use windows::Win32::Graphics::Direct3D::Fxc::D3DCompileFromFile;

pub struct ShaderMacro {
	pub name: String,
	pub definition: String
}

pub struct OsShaderMacro {
	pub name: CString,
	pub definition: CString
}

#[derive(Error, Debug)]
pub enum CompileFromFileToFileError {
	#[error("error during compilation")]
	Compile(#[from] windows::core::Error),
	#[error("error during file io")]
	IO(#[from] std::io::Error)
}

pub fn compile_from_file_to_file<'a, T: IntoParam<'a, ID3DInclude>>(
	input_file_name: String,
	output_file_name: String,
	defines: Option<Vec<ShaderMacro>>,
	include: T,
	entry_point: String,
	target: String,
	compile_flags: CompileFlags,
	effect_compile_flags: EffectCompileFlags
) -> (Result<(), CompileFromFileToFileError>, Option<Vec<u8>>) {
	let (result, messages) = compile_from_file(
		input_file_name,
		defines,
		include,
		entry_point,
		target,
		compile_flags,
		effect_compile_flags
	);
	let result = match result {
		Ok(result) => result,
		Err(err) => {
			return (Err(err.into()), messages)
		}
	};

	let mut file = match File::create(output_file_name) {
		Ok(file) => file,
		Err(err) => {
			return (Err(err.into()), messages)
		}
	};

	match file.write_all(&result) {
		Ok(_) => {
			(Ok(()), messages)
		}
		Err(err) => {
			(Err(err.into()), messages)
		}
	}
}

pub fn compile_from_file<'a, T: IntoParam<'a, ID3DInclude>>(
	file_name: String,
	defines: Option<Vec<ShaderMacro>>,
	include: T,
	entry_point: String,
	target: String,
	compile_flags: CompileFlags,
	effect_compile_flags: EffectCompileFlags
) -> (Result<Vec<u8>, windows::core::Error>, Option<Vec<u8>>) {
	let os_file_name = string_to_os_wide_string(file_name).unwrap();
	let os_entry_point = string_to_os_ascii(entry_point).unwrap();
	let os_target = string_to_os_ascii(target).unwrap();
	let has_defines = defines.is_some();
	let (os_defines, d3d_defines) = match defines {
		Some(defines) => {
			convert_macros(defines).unwrap()
		}
		None => {
			(vec![], vec![])
		}
	};

	let defines_ptr = match has_defines {
		true => {
			d3d_defines.as_ptr()
		}
		false => {
			null()
		}
	};

	let mut code = MaybeUninit::uninit();
	let mut error_messages = MaybeUninit::uninit();

	let result: windows::core::Result<()> = unsafe {
		D3DCompileFromFile(
			PCWSTR(os_file_name.as_ptr()),
			defines_ptr,
			include,
			PCSTR(os_entry_point.as_ptr() as *const u8),
			PCSTR(os_target.as_ptr() as *const u8),
			compile_flags.bits(),
			effect_compile_flags.bits(),
			code.as_mut_ptr(),
			error_messages.as_mut_ptr()
		)
	};

	// Ensure that all the variables live at least until here
	drop(os_file_name);
	drop(os_entry_point);
	drop(os_target);
	drop(d3d_defines);
	drop(os_defines);

	let (code, error_messages) = unsafe {
		(
			code.assume_init(),
			error_messages.assume_init()
		)
	};

	let error_messages = error_messages.map(|item|
		read_blob(item)
	);

	if let Err(err) = result {
		return (Err(err), error_messages);
	}

	let code = match code {
		None => vec![],
		Some(code_blob) => {
			read_blob(code_blob)
		}
	};

	(Ok(code), error_messages)
}

fn read_blob(blob: ID3DBlob) -> Vec<u8> {
	let size = unsafe{
		blob.GetBufferSize()
	};
	let ptr = unsafe {
		blob.GetBufferPointer()
	} as *const u8;

	if ptr.is_null() {
		return vec![];
	}

	let mut vec = Vec::with_capacity(size);

	unsafe {
		copy(ptr, vec.as_mut_ptr(), size);
		vec.set_len(size);
	}

	vec
}

fn convert_macros(macros: Vec<ShaderMacro>) -> Result<(Vec<OsShaderMacro>, Vec<D3D_SHADER_MACRO>), StringToOSAsciiError> {
	let mut os_macros = Vec::with_capacity(macros.len());
	let mut d3d_macros = Vec::with_capacity(macros.len() + 1);

	for shader_macro in macros {
		let (os_macro, d3d_macro) = convert_macro(shader_macro)?;

		os_macros.push(os_macro);
		d3d_macros.push(d3d_macro);
	}

	// The D3D_SHADER_MACRO needs to be terminated by a (NULL, NULL) entry
	// This is because it is passed through FFI as a raw pointer
	d3d_macros.push(D3D_SHADER_MACRO {
		Name: PCSTR(null()),
		Definition: PCSTR(null())
	});

	Ok((os_macros, d3d_macros))
}

fn convert_macro(shader_macro: ShaderMacro) -> Result<(OsShaderMacro, D3D_SHADER_MACRO), StringToOSAsciiError> {
	let os_name = string_to_os_ascii(shader_macro.name)?;
	let os_definition = string_to_os_ascii(shader_macro.definition)?;

	let os_shader_macro = OsShaderMacro {
		name: os_name,
		definition: os_definition
	};
	let d3d_shader_macro = D3D_SHADER_MACRO {
		Name: PCSTR(os_shader_macro.name.as_ptr() as *const u8),
		Definition: PCSTR(os_shader_macro.definition.as_ptr() as *const u8)
	};

	Ok((os_shader_macro, d3d_shader_macro))
}

fn string_to_os_wide_string(string: String) -> Result<U16CString, ContainsNul<u16>> {
	U16CString::from_str(string)
}

#[derive(Error, Debug)]
pub enum StringToOSAsciiError {
	#[error("given string contains non-ascii characters")]
	NonAscii,
	#[error("given string contains interior null bytes")]
	Null(#[from] NulError)
}

fn string_to_os_ascii(string: String) -> Result<CString, StringToOSAsciiError> {
	if !string.is_ascii() {
		return Err(StringToOSAsciiError::NonAscii);
	}
	let string = CString::new(string)?;
	Ok(string)
}


