use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use super::{error::Error, template::ArticleTemplate};

const TEMPLATE_EXTENSION: &str = "typ";

pub(crate) struct TemplateLoader;

impl TemplateLoader {
    pub(crate) fn load_selected(
        template_root: impl AsRef<Path>,
        selected_template: Option<&str>,
    ) -> Result<ArticleTemplate, Error> {
        let template_root = template_root.as_ref();
        let templates = list_templates(template_root)?;
        let template = match selected_template {
            Some(selected_template) => templates
                .into_iter()
                .find(|template| template.name == selected_template)
                .ok_or_else(|| Error::TemplateNotFound {
                    name: selected_template.to_owned(),
                    path: template_root.to_path_buf(),
                })?,
            None => templates
                .into_iter()
                .next()
                .ok_or_else(|| Error::NoTemplates {
                    path: template_root.to_path_buf(),
                })?,
        };
        let contents = fs::read(&template.path).map_err(|source| Error::ReadTemplate {
            path: template.path,
            source,
        })?;

        Ok(ArticleTemplate {
            name: template.name,
            contents,
        })
    }
}

struct TemplateEntry {
    name: String,
    path: PathBuf,
}

fn list_templates(template_root: &Path) -> Result<Vec<TemplateEntry>, Error> {
    let entries = fs::read_dir(template_root).map_err(|source| Error::ReadTemplateDirectory {
        path: template_root.to_path_buf(),
        source,
    })?;

    let mut templates = entries
        .map(|entry| {
            let entry = entry.map_err(|source| Error::ReadTemplateDirectoryEntry {
                path: template_root.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|source| Error::InspectTemplate {
                path: path.clone(),
                source,
            })?;
            if !file_type.is_file()
                || path
                    .extension()
                    .is_none_or(|extension| extension != OsStr::new(TEMPLATE_EXTENSION))
            {
                return Ok(None);
            }

            let name =
                entry
                    .file_name()
                    .into_string()
                    .map_err(|name| Error::InvalidTemplateName {
                        path: template_root.join(name),
                    })?;

            Ok(Some(TemplateEntry { name, path }))
        })
        .collect::<Result<Vec<_>, Error>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    templates.sort_by(|left, right| left.name.cmp(&right.name));

    if templates.is_empty() {
        return Err(Error::NoTemplates {
            path: template_root.to_path_buf(),
        });
    }

    Ok(templates)
}
