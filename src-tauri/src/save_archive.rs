use crate::types::ArchiveFormat;
use anyhow::Context;
use flate2::{write::ZlibEncoder, Compression};
use image::{ColorType, ImageFormat};
use lopdf::{
    content::{Content, Operation},
    dictionary, Dictionary, Document, Object, Stream,
};
use std::path::PathBuf;
use std::{
    io::{Read, Write},
    path::Path,
};

pub fn save_image_archive(
    download_dir: &Path,
    temp_download_dir: &Path,
) -> Result<(), anyhow::Error> {
    if download_dir.exists() {
        std::fs::remove_dir_all(download_dir).context(format!("删除 {download_dir:?} 失败"))?;
    }
    std::fs::rename(temp_download_dir, download_dir).context(format!(
        "将 {temp_download_dir:?} 重命名为 {download_dir:?} 失败"
    ))?;
    Ok(())
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::similar_names)]
pub fn save_pdf_archive(
    download_dir: &Path,
    temp_download_dir: &Path,
    archive_format: &ArchiveFormat,
) -> anyhow::Result<()> {
    let pdf_path = download_dir.with_extension(archive_format.extension());
    let mut doc = Document::with_version("1.5");
    
    let mut image_paths = std::fs::read_dir(temp_download_dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<PathBuf>>();
    image_paths.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    let pages_id = doc.new_object_id();
    let mut page_ids = vec![];
    for path in image_paths {
        // 把图像文件读取到buffer中
        let file = std::fs::File::open(&path).context(format!("打开 {path:?} 失败"))?;
        let mut reader = std::io::BufReader::new(file);
        let mut buffer = vec![];
        reader
            .read_to_end(&mut buffer)
            .context(format!("读取 {path:?} 失败"))?;
        // 识别图像格式
        let format = image::guess_format(&buffer).context("识别 {path:?} 的图像格式失败")?;
        let is_jpeg = format == ImageFormat::Jpeg;
        // 获取图像的颜色类型，如果不是 JPEG 格式，还需要解码图像
        let (img, color_type) = if is_jpeg {
            (None, ColorType::Rgb8) // JPEG 不需要解码
        } else {
            // 其他格式需要解码
            let img = image::load_from_memory(&buffer).context("解码 {path:?} 失败")?;
            let color_type = img.color();
            (Some(img), color_type)
        };
        // 获取图像的尺寸
        let (width, height) =
            image::image_dimensions(&path).context(format!("获取 {path:?} 的尺寸失败"))?;
        // 获取图像每个颜色通道的位数
        let bits = color_type.bits_per_pixel() / 3;
        // 根据颜色类型获取PDF中对应的颜色空间
        let color_space = match color_type {
            ColorType::L8 | ColorType::La8 => b"DeviceGray".to_vec(),
            ColorType::Rgb8 | ColorType::Rgb16 => b"DeviceRGB".to_vec(),
            ColorType::La16 | ColorType::Rgba8 | ColorType::Rgba16 => b"DeviceN".to_vec(),
            _ => b"Indexed".to_vec(),
        };

        let mut dict = Dictionary::new();
        dict.set("Type", Object::Name(b"XObject".to_vec()));
        dict.set("Subtype", Object::Name(b"Image".to_vec()));
        dict.set("Width", width);
        dict.set("Height", height);
        dict.set("ColorSpace", Object::Name(color_space));
        dict.set("BitsPerComponent", bits);
        // 如果图像是 JPEG 格式，需要设置滤波器为 DCTDecode
        let image_stream = if is_jpeg {
            dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
            Stream::new(dict, buffer)
        } else {
            let Some(img) = img else { unreachable!() };
            Stream::new(dict, img.into_bytes())
        };
        // 将图像添加到文档中
        let img_id = doc.add_object(image_stream);
        // 图像的名称，用于 Do 操作在页面上显示图像
        let img_name = format!("X{}", img_id.0);
        let (width, height) =
            image::image_dimensions(&path).context(format!("获取 {path:?} 的尺寸失败"))?;
        // 用于设置图像在页面上的位置和大小
        let cm_operation = Operation::new(
            "cm",
            vec![
                width.into(),
                0.into(),
                0.into(),
                height.into(),
                0.into(),
                0.into(),
            ],
        );
        // 用于显示图像
        let do_operation = Operation::new("Do", vec![Object::Name(img_name.as_bytes().to_vec())]);
        // 创建页面，设置图像的位置和大小，然后显示图像
        // 因为是从零开始创建文档，所以没必要用 q 和 Q 操作保存和恢复图形状态
        let content = Content {
            operations: vec![cm_operation, do_operation],
        };
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.encode()?));
        let page_id = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "Contents" => content_id,
            "MediaBox" => vec![0.into(), 0.into(), width.into(), height.into()],
        });
        // 将图像以 XObject 的形式添加到文档中
        // Do 操作只能引用 XObject(所以前面定义的 Do 操作的参数是 img_name, 而不是 img_id)
        doc.add_xobject(page_id, img_name.as_bytes(), img_id)?;
        // 记录新创建的页面的 ID
        page_ids.push(page_id);
    }

    let count = page_ids.len() as u32;
    let pages = dictionary! {
        "Type" => "Pages",
        "Kids" => page_ids.into_iter()
                    .map(Object::Reference)
                    .collect::<Vec<_>>(),
        "Count" => count,
    };

    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(dictionary! {
        "Type" => "Catalog",
        "Pages" => pages_id,
    });

    doc.trailer.set("Root", catalog_id);
    // 压缩所有未压缩的流
    compress_pdf(&mut doc).context("压缩 {pdf_path:?} 失败")?;

    doc.save(&pdf_path)
        .context(format!("保存 {pdf_path:?} 失败"))?;

    std::fs::remove_dir_all(temp_download_dir)
        .context(format!("删除 {temp_download_dir:?} 失败"))?;
    Ok(())
}

fn compress_pdf(doc: &mut Document) -> anyhow::Result<()> {
    for object in doc.objects.values_mut() {
        // 只压缩流对象
        let Object::Stream(ref mut stream) = *object else {
            continue;
        };
        // 只压缩允许压缩的流对象
        if !stream.allows_compression {
            continue;
        }
        // 如果已经压缩，跳过
        if stream.dict.get(b"Filter").is_ok() {
            continue;
        }
        // 压缩流对象
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::fast());
        encoder.write_all(stream.content.as_slice())?;
        let compressed = encoder.finish()?;
        if compressed.len() + 19 < stream.content.len() {
            stream.dict.set("Filter", "FlateDecode");
            stream.set_content(compressed);
        }
    }
    Ok(())
}
