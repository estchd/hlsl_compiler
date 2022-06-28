use bitflags::bitflags;
use windows::Win32::Graphics::Direct3D::Fxc::{
	D3DCOMPILE_EFFECT_CHILD_EFFECT,
	D3DCOMPILE_EFFECT_ALLOW_SLOW_OPS
};

bitflags! {
	pub struct EffectCompileFlags: u32 {
		const CHILD_EFFECT = D3DCOMPILE_EFFECT_CHILD_EFFECT;
		const ALLOW_SLOW_OPS = D3DCOMPILE_EFFECT_ALLOW_SLOW_OPS;
	}
}