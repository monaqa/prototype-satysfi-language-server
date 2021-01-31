//! 2つの区間同士の関係を表す Trait.

/// 2つの区間同士の関係を表す Trait. Range に実装する。
pub trait CompareRange: Sized {

    /// 自身が対象を include しているか。
    fn includes(&self, other: &Self) -> bool;

    /// 自身が対象に include されているか。
    fn is_included(&self, other: &Self) -> bool;

    /// 共通部分。交わりがなければ None を返す。
    fn intersect(&self, other: &Self) -> Option<Self>;

    /// 共通部分が存在するかどうか。
    fn has_intersect(&self, other: &Self) -> bool;

}
