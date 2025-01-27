use std::{array, marker::PhantomData, mem::size_of};

use bytemuck::Pod;

use crate::flver::vertex_buffer::normalization::{
    NoNormalization, SNorm, UNorm, VertexAttributeNormalization,
};

pub enum VertexAttributeAccessor<'a> {
    Float2(VertexAttributeIter<'a, f32, 2>),
    Float3(VertexAttributeIter<'a, f32, 3>),
    Float4(VertexAttributeIter<'a, f32, 4>),
    UNorm8x4(VertexAttributeIter<'a, u8, 4, UNorm<u8, 255>>),
    UNorm4x4(VertexAttributeIter<'a, u8, 4, UNorm<u8, 127>>),
    UNorm16x2(VertexAttributeIter<'a, u16, 2, UNorm<u16, 32767>>),
    UNorm16x4(VertexAttributeIter<'a, u16, 4, UNorm<u16, 32767>>),
    SNorm8x4(VertexAttributeIter<'a, u8, 4, SNorm<u8, 127>>),
    SNorm16x4(VertexAttributeIter<'a, u16, 4, SNorm<u16, 32767>>),
    SNorm16x2(VertexAttributeIter<'a, u16, 2, SNorm<u16, 32767>>),
    UV(VertexAttributeIter<'a, f32, 2>),
    // TODO: get the last 2 components of this
    UVPair(VertexAttributeIter<'a, f32, 2>),
    Short4ToFloat4A(VertexAttributeIter<'a, u16, 4>),
    Short4ToFloat4B(VertexAttributeIter<'a, u16, 4>),
}

pub struct VertexAttributeIter<
    'a,
    T: Pod,
    const L: usize,
    N: VertexAttributeNormalization = NoNormalization<T>,
> {
    buffer: &'a [u8],
    attribute_data_offset: usize,
    attribute_data_end: usize,
    vertex_size: usize,
    _value: PhantomData<T>,
    _normalization: PhantomData<N>,
}

// TODO: this doesn't support endian sensitive reading like the rest of the FLVER parser.
impl<'a, T: Pod, const L: usize, N: VertexAttributeNormalization> VertexAttributeIter<'a, T, L, N> {
    pub fn new(
        buffer: &'a [u8],
        vertex_size: usize,
        vertex_offset: usize,
    ) -> VertexAttributeIter<'a, T, L, N> {
        let attribute_data_offset = vertex_offset;
        let attribute_data_end = attribute_data_offset + size_of::<T>() * L;

        Self {
            buffer,
            attribute_data_offset,
            attribute_data_end,
            vertex_size,
            _value: PhantomData,
            _normalization: PhantomData,
        }
    }

    pub fn no_norm(self) -> VertexAttributeIter<'a, T, L, NoNormalization<T>> {
        let Self {
            buffer,
            vertex_size,
            attribute_data_offset,
            ..
        } = self;

        VertexAttributeIter::new(buffer, vertex_size, attribute_data_offset)
    }
}

impl<'a, T: Pod, const L: usize, N: VertexAttributeNormalization<Input = T>> ExactSizeIterator
    for VertexAttributeIter<'a, T, L, N>
{
}
impl<'a, T: Pod, const L: usize, N: VertexAttributeNormalization<Input = T>> Iterator
    for VertexAttributeIter<'a, T, L, N>
{
    type Item = [N::Output; L];

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            return None;
        }

        let attribute_byte_data = &self.buffer[self.attribute_data_offset..self.attribute_data_end];
        let data: &[T] = bytemuck::cast_slice(attribute_byte_data);
        let output: [N::Output; L] = array::from_fn(|index| N::normalize(&data[index]));

        self.buffer = &self.buffer[self.vertex_size..];

        Some(output)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.buffer.len() / self.vertex_size;
        (remaining, Some(remaining))
    }
}
