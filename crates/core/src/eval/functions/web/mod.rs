use super::Registry;
use super::FunctionMeta;

pub mod encodeurl;
pub mod hyperlink;
pub mod isurl;

pub fn register_web(registry: &mut Registry) {
    registry.register_eager("ENCODEURL", encodeurl::encodeurl_fn, FunctionMeta { category: "web", signature: "ENCODEURL(url)",               description: "Percent-encode a URL string per RFC 3986" });
    registry.register_eager("HYPERLINK", hyperlink::hyperlink_fn, FunctionMeta { category: "web", signature: "HYPERLINK(url, [link_label])", description: "Return link label (or url if no label)" });
    registry.register_eager("ISURL",     isurl::isurl_fn,         FunctionMeta { category: "web", signature: "ISURL(value)",                 description: "Return TRUE if value is a URL string" });
}
