// Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

use std::borrow::Cow;
use std::marker::PhantomData;
use std::mem;
use std::ops::Range;
use std::ptr;
use std::sync::Arc;
use std::ffi::CStr;
use std::vec::IntoIter as VecIntoIter;

use format::Format;
use pipeline::input_assembly::PrimitiveTopology;

use device::Device;
use OomError;
use VulkanObject;
use VulkanPointers;
use check_errors;
use vk;

/// Contains SPIR-V code with one or more entry points.
///
/// Note that it is advised to wrap around a `ShaderModule` with a struct that is different for
/// each shader.
pub struct ShaderModule {
    device: Arc<Device>,
    module: vk::ShaderModule,
}

impl ShaderModule {
    /// Builds a new shader module from SPIR-V.
    ///
    /// # Safety
    ///
    /// - The SPIR-V code is not validated.
    /// - The SPIR-V code may require some features that are not enabled. This isn't checked by
    ///   this function either.
    ///
    pub unsafe fn new(device: &Arc<Device>, spirv: &[u8])
                      -> Result<Arc<ShaderModule>, OomError>
    {
        let vk = device.pointers();

        assert!((spirv.len() % 4) == 0);

        let module = {
            let infos = vk::ShaderModuleCreateInfo {
                sType: vk::STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
                pNext: ptr::null(),
                flags: 0,   // reserved
                codeSize: spirv.len(),
                pCode: spirv.as_ptr() as *const _,
            };

            let mut output = mem::uninitialized();
            try!(check_errors(vk.CreateShaderModule(device.internal_object(), &infos,
                                                    ptr::null(), &mut output)));
            output
        };

        Ok(Arc::new(ShaderModule {
            device: device.clone(),
            module: module,
        }))
    }

    pub unsafe fn vertex_shader_entry_point<'a, S, I, O, L>
        (&'a self, name: &'a CStr, input: I, output: O, layout: L)
        -> VertexShaderEntryPoint<'a, S, I, O, L>
    {
        VertexShaderEntryPoint {
            module: self,
            name: name,
            input: input,
            output: output,
            layout: layout,
            marker: PhantomData,
        }
    }

    pub unsafe fn tess_control_shader_entry_point<'a, S, I, O, L>
        (&'a self, name: &'a CStr, input: I, output: O, layout: L) -> TessControlShaderEntryPoint<'a, S, I, O, L>
    {
        TessControlShaderEntryPoint {
            module: self,
            name: name,
            layout: layout,
            input: input,
            output: output,
            marker: PhantomData,
        }
    }

    pub unsafe fn tess_evaluation_shader_entry_point<'a, S, I, O, L>
        (&'a self, name: &'a CStr, input: I, output: O, layout: L) -> TessEvaluationShaderEntryPoint<'a, S, I, O, L>
    {
        TessEvaluationShaderEntryPoint {
            module: self,
            name: name,
            layout: layout,
            input: input,
            output: output,
            marker: PhantomData,
        }
    }

    pub unsafe fn geometry_shader_entry_point<'a, S, I, O, L>
        (&'a self, name: &'a CStr, primitives: GeometryShaderExecutionMode, input: I, output: O, layout: L)
        -> GeometryShaderEntryPoint<'a, S, I, O, L>
    {
        GeometryShaderEntryPoint {
            module: self,
            name: name,
            layout: layout,
            primitives: primitives,
            input: input,
            output: output,
            marker: PhantomData,
        }
    }

    /// Gets access to an entry point contained in this module.
    ///
    /// This is purely a *logical* operation. It returns a struct that *represents* the entry
    /// point but doesn't actually do anything.
    ///
    /// # Safety
    ///
    /// - The user must check that the entry point exists in the module, as this is not checked
    ///   by Vulkan.
    /// - Calling this function also determines the template parameters associated to the
    ///   `EntryPoint` struct. Therefore care must be taken that the values there are correct.
    ///
    pub unsafe fn fragment_shader_entry_point<'a, S, I, O, L>(&'a self, name: &'a CStr, input: I, output: O, layout: L)
                                                              -> FragmentShaderEntryPoint<'a, S, I, O, L>
    {
        FragmentShaderEntryPoint {
            module: self,
            name: name,
            layout: layout,
            input: input,
            output: output,
            marker: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn compute_shader_entry_point<'a, S, L>(&'a self, name: &'a CStr, layout: L)
                                                       -> ComputeShaderEntryPoint<'a, S, L>
    {
        ComputeShaderEntryPoint {
            module: self,
            name: name,
            layout: layout,
            marker: PhantomData,
        }
    }
}

unsafe impl VulkanObject for ShaderModule {
    type Object = vk::ShaderModule;

    #[inline]
    fn internal_object(&self) -> vk::ShaderModule {
        self.module
    }
}

impl Drop for ShaderModule {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            let vk = self.device.pointers();
            vk.DestroyShaderModule(self.device.internal_object(), self.module, ptr::null());
        }
    }
}

pub struct VertexShaderEntryPoint<'a, S, I, O, L> {
    module: &'a ShaderModule,
    name: &'a CStr,
    input: I,
    layout: L,
    output: O,
    marker: PhantomData<S>,
}

impl<'a, S, I, O, L> VertexShaderEntryPoint<'a, S, I, O, L> {
    #[inline]
    pub fn module(&self) -> &'a ShaderModule {
        self.module
    }

    #[inline]
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    #[inline]
    pub fn layout(&self) -> &L {
        &self.layout
    }

    // TODO: rename "input" for consistency
    #[inline]
    pub fn input_definition(&self) -> &I {
        &self.input
    }

    #[inline]
    pub fn output(&self) -> &O {
        &self.output
    }
}

pub struct TessControlShaderEntryPoint<'a, S, I, O, L> {
    module: &'a ShaderModule,
    name: &'a CStr,
    layout: L,
    input: I,
    output: O,
    marker: PhantomData<S>,
}

impl<'a, S, I, O, L> TessControlShaderEntryPoint<'a, S, I, O, L> {
    #[inline]
    pub fn module(&self) -> &'a ShaderModule {
        self.module
    }

    #[inline]
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    #[inline]
    pub fn layout(&self) -> &L {
        &self.layout
    }

    #[inline]
    pub fn input(&self) -> &I {
        &self.input
    }

    #[inline]
    pub fn output(&self) -> &O {
        &self.output
    }
}

pub struct TessEvaluationShaderEntryPoint<'a, S, I, O, L> {
    module: &'a ShaderModule,
    name: &'a CStr,
    layout: L,
    input: I,
    output: O,
    marker: PhantomData<S>,
}

impl<'a, S, I, O, L> TessEvaluationShaderEntryPoint<'a, S, I, O, L> {
    #[inline]
    pub fn module(&self) -> &'a ShaderModule {
        self.module
    }

    #[inline]
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    #[inline]
    pub fn layout(&self) -> &L {
        &self.layout
    }

    #[inline]
    pub fn input(&self) -> &I {
        &self.input
    }

    #[inline]
    pub fn output(&self) -> &O {
        &self.output
    }
}

pub struct GeometryShaderEntryPoint<'a, S, I, O, L> {
    module: &'a ShaderModule,
    name: &'a CStr,
    layout: L,
    primitives: GeometryShaderExecutionMode,
    input: I,
    output: O,
    marker: PhantomData<S>,
}

impl<'a, S, I, O, L> GeometryShaderEntryPoint<'a, S, I, O, L> {
    #[inline]
    pub fn module(&self) -> &'a ShaderModule {
        self.module
    }

    #[inline]
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    /// Returns the kind of primitives expected by the geometry shader.
    #[inline]
    pub fn primitives(&self) -> GeometryShaderExecutionMode {
        self.primitives
    }

    #[inline]
    pub fn layout(&self) -> &L {
        &self.layout
    }

    #[inline]
    pub fn input(&self) -> &I {
        &self.input
    }

    #[inline]
    pub fn output(&self) -> &O {
        &self.output
    }
}

/// Declares which type of primitives are expected by the geometry shader.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GeometryShaderExecutionMode {
    Points,
    Lines,
    LinesWithAdjacency,
    Triangles,
    TrianglesWithAdjacency,
}

impl GeometryShaderExecutionMode {
    /// Returns true if the given primitive topology can be used with this execution mode.
    #[inline]
    pub fn matches(&self, input: PrimitiveTopology) -> bool {
        match (*self, input) {
            (GeometryShaderExecutionMode::Points, PrimitiveTopology::PointList) => true,
            (GeometryShaderExecutionMode::Lines, PrimitiveTopology::LineList) => true,
            (GeometryShaderExecutionMode::Lines, PrimitiveTopology::LineStrip) => true,
            (GeometryShaderExecutionMode::LinesWithAdjacency,
             PrimitiveTopology::LineListWithAdjacency) => true,
            (GeometryShaderExecutionMode::LinesWithAdjacency,
             PrimitiveTopology::LineStripWithAdjacency) => true,
            (GeometryShaderExecutionMode::Triangles, PrimitiveTopology::TriangleList) => true,
            (GeometryShaderExecutionMode::Triangles, PrimitiveTopology::TriangleStrip) => true,
            (GeometryShaderExecutionMode::Triangles, PrimitiveTopology::TriangleFan) => true,
            (GeometryShaderExecutionMode::TrianglesWithAdjacency,
             PrimitiveTopology::TriangleListWithAdjancecy) => true,
            (GeometryShaderExecutionMode::TrianglesWithAdjacency,
             PrimitiveTopology::TriangleStripWithAdjacency) => true,
            _ => false,
        }
    }
}

pub struct FragmentShaderEntryPoint<'a, S, I, O, L> {
    module: &'a ShaderModule,
    name: &'a CStr,
    layout: L,
    input: I,
    output: O,
    marker: PhantomData<S>,
}

impl<'a, S, I, O, L> FragmentShaderEntryPoint<'a, S, I, O, L> {
    #[inline]
    pub fn module(&self) -> &'a ShaderModule {
        self.module
    }

    #[inline]
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    #[inline]
    pub fn layout(&self) -> &L {
        &self.layout
    }

    #[inline]
    pub fn input(&self) -> &I {
        &self.input
    }

    #[inline]
    pub fn output(&self) -> &O {
        &self.output
    }
}

pub struct ComputeShaderEntryPoint<'a, S, L> {
    module: &'a ShaderModule,
    name: &'a CStr,
    layout: L,
    marker: PhantomData<S>,
}

impl<'a, S, L> ComputeShaderEntryPoint<'a, S, L> {
    #[inline]
    pub fn module(&self) -> &'a ShaderModule {
        self.module
    }

    #[inline]
    pub fn name(&self) -> &'a CStr {
        self.name
    }

    #[inline]
    pub fn layout(&self) -> &L {
        &self.layout
    }
}

/// Structs that contain the definition of an interface between two shader stages, or between
/// the outside and a shader stage.
///
/// # Safety
///
/// - Must only provide one entry per location.
/// - The format must not be larger than 128 bits.
///
pub unsafe trait ShaderInterfaceDef {
    /// Iterator returned by `elements`.
    type Iter: ExactSizeIterator<Item = ShaderInterfaceDefEntry>;

    /// Iterates over the elements of the interface.
    fn elements(&self) -> Self::Iter;
}

// FIXME: temporary ; remove as it is unsafe
unsafe impl ShaderInterfaceDef for Vec<ShaderInterfaceDefEntry> {
    type Iter = VecIntoIter<ShaderInterfaceDefEntry>;

    #[inline]
    fn elements(&self) -> Self::Iter {
        self.clone().into_iter()
    }
}

/// Entry of a shader interface definition.
#[derive(Debug, Clone)]
pub struct ShaderInterfaceDefEntry {
    /// Range of locations covered by the element.
    pub location: Range<u32>,
    /// Format of a each location of the element.
    pub format: Format,
    /// Name of the element, or `None` if the name is unknown.
    pub name: Option<Cow<'static, str>>,
}

/// Extension trait for `ShaderInterfaceDef` that specifies that the interface is potentially
/// compatible with another one.
pub unsafe trait ShaderInterfaceDefMatch<I>: ShaderInterfaceDef where I: ShaderInterfaceDef {
    /// Returns true if the two definitions match.
    // TODO: return a descriptive error instead
    fn matches(&self, other: &I) -> bool;
}

// TODO: turn this into a default impl that can be specialized
unsafe impl<T, I> ShaderInterfaceDefMatch<I> for T
    where T: ShaderInterfaceDef, I: ShaderInterfaceDef
{
    fn matches(&self, other: &I) -> bool {
        if self.elements().len() != other.elements().len() {
            return false;
        }

        for a in self.elements() {
            for loc in a.location.clone() {
                let b = match other.elements().find(|e| loc >= e.location.start && loc < e.location.end) {
                    None => return false,
                    Some(b) => b,
                };

                if a.format != b.format {
                    return false;
                }

                // TODO: enforce this?
                /*match (a.name, b.name) {
                    (Some(ref an), Some(ref bn)) => if an != bn { return false },
                    _ => ()
                };*/
            }
        }

        true
    }
}

/// Trait to describe structs that contain specialization data for shaders.
///
/// It is implemented on `()` for shaders that don't have any specialization constant.
pub unsafe trait SpecializationConstants {
    /// Returns descriptors of the struct's layout.
    fn descriptors() -> &'static [SpecializationMapEntry];
}

unsafe impl SpecializationConstants for () {
    #[inline]
    fn descriptors() -> &'static [SpecializationMapEntry] {
        &[]
    }
}

/// Describes an indiviual constant to set in the shader. Also a field in the struct.
// Has the same memory representation as a `VkSpecializationMapEntry`.
#[repr(C)]
pub struct SpecializationMapEntry {
    /// Identifier of the constant in the shader that corresponds to this field.
    pub constant_id: u32,
    /// Offset within this struct for the data.
    pub offset: u32,
    /// Size of the data in bytes.
    pub size: usize,
}
