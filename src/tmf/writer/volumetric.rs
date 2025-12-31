use std::io::Write;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use super::error::Result;
use super::define::volumetric::*;

impl Function {
    /// Write function to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        // Write function type
        match &self.r#type {
            FunctionType::FunctionFromImage3D(func) => {
                let mut elem = FunctionFromImage3D::start();
                elem.push_attribute(("id", self.id.to_string().as_str()));
                if let Some(ref display_name) = self.display_name {
                    elem.push_attribute(("displayname", display_name.as_str()));
                }
                func.write(elem, writer)?;
            },
            FunctionType::PrivateExtensionFunction(func) => func.write(writer)?,
        }

        Ok(())
    }
}

impl FunctionFromImage3D {
    fn start<'a>() -> BytesStart<'a> {
        BytesStart::new("v:functionfromimage3d")
    }

    /// Write functionfromimage3d to XML.
    fn write<W: Write>(&self, mut elem: BytesStart, writer: &mut Writer<W>) -> Result<()> {
        elem.push_attribute(("image3did", self.image_3d_id.to_string().as_str()));

        if self.value_offset != 0.0 {
            elem.push_attribute(("valueoffset", self.value_offset.to_string().as_str()));
        }

        if self.value_scale != 1.0 {
            elem.push_attribute(("valuescale", self.value_scale.to_string().as_str()));
        }

        if self.filter != Filter::Linear {
            elem.push_attribute(("filter", self.filter.to_string().as_str()));
        }

        if self.tile_style_u != TileStyle::Wrap {
            elem.push_attribute(("tilestyleu", self.tile_style_u.to_string().as_str()));
        }

        if self.tile_style_v != TileStyle::Wrap {
            elem.push_attribute(("tilestylev", self.tile_style_v.to_string().as_str()));
        }

        if self.tile_style_w != TileStyle::Wrap {
            elem.push_attribute(("tilestylew", self.tile_style_w.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl PrivateExtensionFunction {
    /// Write private extension function to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("privateextensionfunction");
        for (key, value) in &self.attributes {
            elem.push_attribute((key.as_str(), value.as_str()));
        }
        if let Some(content) = &self.content {
            writer.write_event(Event::Text(BytesText::new(content.as_str())))?;
        }
        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl Image3D {
    /// Write image3d to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:image3d");
        elem.push_attribute(("id", self.id.to_string().as_str()));

        if let Some(ref name) = self.name {
            elem.push_attribute(("name", name.as_str()));
        }

        writer.write_event(Event::Start(elem))?;

        // Write content
        match &self.content {
            Image3DContent::ImageStack(stack) => stack.write(writer)?,
        }

        writer.write_event(Event::End(BytesEnd::new("v:image3d")))?;

        Ok(())
    }
}

impl ImageStack {
    /// Write imagestack to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:imagestack");
        elem.push_attribute(("rowcount", self.row_count.to_string().as_str()));
        elem.push_attribute(("columncount", self.column_count.to_string().as_str()));
        elem.push_attribute(("sheetcount", self.sheet_count.to_string().as_str()));

        writer.write_event(Event::Start(elem))?;

        // Write image sheets
        for sheet in &self.image_sheets {
            sheet.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("v:imagestack")))?;

        Ok(())
    }
}

impl ImageSheet {
    /// Write imagesheet to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:imagesheet");
        elem.push_attribute(("path", self.path.as_str()));

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl VolumeData {
    /// Write volumedata to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:volumedata");
        elem.push_attribute(("id", self.id.to_string().as_str()));

        writer.write_event(Event::Start(elem))?;

        // Write color if present
        if let Some(ref color) = self.color {
            color.write(writer)?;
        }

        // Write composite if present
        if let Some(ref composite) = self.composite {
            composite.write(writer)?;
        }

        // Write properties
        for property in &self.properties {
            property.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("v:volumedata")))?;

        Ok(())
    }
}

impl Colour {
    /// Write color to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:color");
        elem.push_attribute(("functionid", self.function_id.to_string().as_str()));
        elem.push_attribute(("channel", self.channel.as_str()));

        if let Some(ref transform) = self.transform {
            elem.push_attribute(("transform", transform.to_string().as_str()));
        }

        if self.min_feature_size != 0.0 {
            elem.push_attribute(("minfeaturesize", self.min_feature_size.to_string().as_str()));
        }

        if self.fallback_value != 0.0 {
            elem.push_attribute(("fallbackvalue", self.fallback_value.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl Composite {
    /// Write composite to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:composite");
        elem.push_attribute(("basematerialid", self.base_material_id.to_string().as_str()));

        writer.write_event(Event::Start(elem))?;

        // Write material mappings
        for mapping in &self.material_mappings {
            mapping.write(writer)?;
        }

        writer.write_event(Event::End(BytesEnd::new("v:composite")))?;

        Ok(())
    }
}

impl MaterialMapping {
    /// Write materialmapping to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:materialmapping");
        elem.push_attribute(("functionid", self.function_id.to_string().as_str()));
        elem.push_attribute(("channel", self.channel.as_str()));

        if let Some(ref transform) = self.transform {
            elem.push_attribute(("transform", transform.to_string().as_str()));
        }

        if self.min_feature_size != 0.0 {
            elem.push_attribute(("minfeaturesize", self.min_feature_size.to_string().as_str()));
        }

        if self.fallback_value != 0.0 {
            elem.push_attribute(("fallbackvalue", self.fallback_value.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl Property {
    /// Write property to XML.
    fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:property");
        elem.push_attribute(("functionid", self.function_id.to_string().as_str()));
        elem.push_attribute(("channel", self.channel.as_str()));
        elem.push_attribute(("name", self.name.as_str()));

        if let Some(ref transform) = self.transform {
            elem.push_attribute(("transform", transform.to_string().as_str()));
        }

        if self.required {
            elem.push_attribute(("required", "1"));
        }

        if self.min_feature_size != 0.0 {
            elem.push_attribute(("minfeaturesize", self.min_feature_size.to_string().as_str()));
        }

        if self.fallback_value != 0.0 {
            elem.push_attribute(("fallbackvalue", self.fallback_value.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl LevelSet {
    /// Write levelset to XML.
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        let mut elem = BytesStart::new("v:levelset");
        elem.push_attribute(("functionid", self.function_id.to_string().as_str()));
        elem.push_attribute(("channel", self.channel.as_str()));
        elem.push_attribute(("meshid", self.mesh_id.to_string().as_str()));

        if let Some(ref transform) = self.transform {
            elem.push_attribute(("transform", transform.to_string().as_str()));
        }

        if self.min_feature_size != 0.0 {
            elem.push_attribute(("minfeaturesize", self.min_feature_size.to_string().as_str()));
        }

        if self.mesh_bbox_only {
            elem.push_attribute(("meshbboxonly", "1"));
        }

        if self.fallback_value != 0.0 {
            elem.push_attribute(("fallbackvalue", self.fallback_value.to_string().as_str()));
        }

        if let Some(volume_id) = self.volume_id {
            elem.push_attribute(("volumeid", volume_id.to_string().as_str()));
        }

        writer.write_event(Event::Empty(elem))?;

        Ok(())
    }
}

impl VolumetricResources {
    /// Write volumetric resources to XML.
    pub fn write<W: Write>(&self, writer: &mut Writer<W>) -> Result<()> {
        // Write functions
        for function in &self.functions {
            function.write(writer)?;
        }

        // Write image3ds
        for image3d in &self.image_3ds {
            image3d.write(writer)?;
        }

        // Write volumedatas
        for volumedata in &self.volume_datas {
            volumedata.write(writer)?;
        }

        Ok(())
    }
}