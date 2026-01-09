use std::io::{Seek, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, Event};
use quick_xml::Writer;
use crate::util::path::normalize;
use super::error::Result;
use super::define::package::*;

impl ContentTypes {
    fn write<W: Write>(self: &Self, writer: W) -> Result<()> {
        let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);
        xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut types_elem = BytesStart::new("Types");
        types_elem.push_attribute(("xmlns", Namespace::CONTENT_TYPES_NS));
        xml_writer.write_event(Event::Start(types_elem))?;

        if self.defaults.is_empty() {
            let mut rels_elem = BytesStart::new("Default");
            rels_elem.push_attribute(("Extension", Extension::RELATIONSHIP_EXT));
            rels_elem.push_attribute(("ContentType", ContentType::RELATIONSHIP_CT));
            xml_writer.write_event(Event::Empty(rels_elem))?;
            let mut model_elem = BytesStart::new("Default");
            model_elem.push_attribute(("Extension", Extension::MODEL_EXT));
            model_elem.push_attribute(("ContentType", ContentType::MODEL_CT));
            xml_writer.write_event(Event::Empty(model_elem))?;
        } else {
            for default_types in &self.defaults {
                let mut default_elem = BytesStart::new("Default");
                default_elem.push_attribute(("Extension", default_types.extension.as_str()));
                default_elem.push_attribute(("ContentType", default_types.content_type.as_str()));
                xml_writer.write_event(Event::Empty(default_elem))?;
            }
        }

        for override_types in &self.overrides {
            let mut override_elem = BytesStart::new("Override");
            override_elem.push_attribute(("PartName", override_types.part_name.as_str()));
            override_elem.push_attribute(("ContentType", override_types.content_type.as_str()));
            xml_writer.write_event(Event::Empty(override_elem))?;
        }

        for (unknown_key, unknown_types) in &self.unknowns {
            for unknown_type in unknown_types {
                let mut unknown_elem = BytesStart::new(unknown_key);
                for (unknown_field, unknown_value) in unknown_type {
                    unknown_elem.push_attribute((unknown_field.as_str(), unknown_value.as_str()));
                }
                xml_writer.write_event(Event::Empty(unknown_elem))?;
            }
        }

        xml_writer.write_event(Event::End(BytesEnd::new("Types")))?;

        Ok(())
    }
}

impl Relationships {
    fn write<W: Write>(self: &Self, writer: W) -> Result<()> {
        let mut xml_writer = Writer::new_with_indent(writer, b' ', 2);
        xml_writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;

        let mut rels_elem = BytesStart::new("Relationships");
        rels_elem.push_attribute(("xmlns", Namespace::RELATIONSHIP_NS));
        xml_writer.write_event(Event::Start(rels_elem))?;

        for rel in &self.relationships {
            let mut rel_elem = BytesStart::new("Relationship");
            rel_elem.push_attribute(("Id", rel.id.as_str()));
            rel_elem.push_attribute(("Type", rel.r#type.as_str()));
            rel_elem.push_attribute(("Target", rel.target.as_str()));
            xml_writer.write_event(Event::Empty(rel_elem))?;
        }

        xml_writer.write_event(Event::End(BytesEnd::new("Relationships")))?;

        Ok(())
    }
}

impl Package {
    pub fn write<W: Write + Seek>(self: &Self, writer: W) -> Result<()> {
        let mut zip = ZipWriter::new(writer);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // Content types mutable for some write
        let mut content_types = self.content_types.clone();
        // Relationships mutable for some write
        let mut relationships = self.relationships.clone();

        // Write 3D model
        let model_path = normalize(Pathway::MODEL_PATH, None);
        zip.start_file(&model_path, options)?;
        self.model.write(&mut zip)?;

        // Write thumbnail if present
        let mut thumb_rels = None;
        if let Some(ref thumbnail) = self.thumbnail {
            thumb_rels = Some((thumbnail.content_type.clone(), thumbnail.extension.clone()));
            let thumb_path = normalize(&(Pathway::THUMBNAIL_PATH.to_owned() + "." + &thumbnail.extension), None);
            zip.start_file(&thumb_path, options)?;
            zip.write_all(&thumbnail.data)?;
        }

        // Check root relationships
        if relationships.is_empty() {
            relationships.add_relationship(Relationship {
                id: relationships.next_id(),
                r#type: Namespace::MODEL_NS.to_string(),
                target: Pathway::MODEL_PATH.to_string(),
            });
            if let Some((thumb_ns, thumb_ext)) = thumb_rels {
                relationships.add_relationship(Relationship {
                    id: relationships.next_id(),
                    r#type: thumb_ns,
                    target: thumb_ext,
                });
            }
        }

        // Write other relationships
        for (path, rels) in &self.relative_rels {
            if path == Pathway::RELS_ROOT_PATH {
                continue;
            }
            zip.start_file(normalize(path, None), options)?;
            rels.write(&mut zip)?;
        }

        // Write other models
        for (path, model) in &self.models {
            zip.start_file(normalize(path, None), options)?;
            model.write(&mut zip)?;
        }

        // Write attachments
        for (path, attach) in &self.attachments {
            let attach_path = normalize(path, None);
            zip.start_file(&attach_path, options)?;
            zip.write_all(&attach.data)?;
        }

        // Write key store for secure content extension
        if let Some(ref keystore) = self.key_store {
            zip.start_file(normalize(Pathway::KEY_STORE_PATH, None), options)?;
            keystore.write(&mut zip)?;
            relationships.ensure(Namespace::KEY_STORE_NS, Pathway::KEY_STORE_PATH);
            content_types.ensure_override(Pathway::KEY_STORE_PATH, ContentType::KEY_STORE_CT);
        }

        // Write root relationships
        zip.start_file(normalize(Pathway::RELS_ROOT_PATH, None), options)?;
        relationships.write(&mut zip)?;

        // Write content types
        zip.start_file(normalize(Pathway::CONTENT_TYPES_PATH, None), options)?;
        content_types.write(&mut zip)?;

        zip.finish()?;

        Ok(())
    }
}