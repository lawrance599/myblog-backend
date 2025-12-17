use std::sync::LazyLock;

pub static MARKDOWN_UTIL: LazyLock<MarkdownUtil> = LazyLock::new(|| MarkdownUtil::new());
/// 用来处理markdown文件的工具函数集合
/// - 获取markdown文件的front matter (使用grey_matter)
/// - 将markdown文件转为纯文本 (使用texting)
/// - 将markdown文件内容使用嵌入模型转化为向量(使用硅基流动)
//TODO
pub struct MarkdownUtil {}

impl MarkdownUtil {
    fn new() -> Self {
        Self {}
    }
}
