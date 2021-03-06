# DoreaDB Design

> 关于 `DoreaDB` 设计时的一些构思。

## 最大索引数量 [2021-12-14 写]

这是一个及其难控制的系统，因为我们的数据库需要一套合理的缓存淘汰机制，而且淘汰数据索引需要以 `Group 数据库` 作为对象。
也就是说当内存满了之后，我们不能单单淘汰最不常使用的那个键（根据LRU淘汰算法），而是要淘汰最不常使用的那一整个数据库。
所以说索引数的管理是非常重要的，目前打算进行的设计是：单个数据库能有键数占最大总支持大小的四分之一，即最大为102400时，单个库最多20480；

索引系统会自动根据使用状态进行管理，并自动清除长期未使用的数据库。

### 权重管理 [2021-12-21 追]

具体请前往作者博客查看：[mrxzx.info](https://mrxzx.info/)

## Fuzzy Search 设计方案 [2021-12-18 写]

目前我正在设计系统中的 `模糊匹配` 功能，它适用于 Key 和 Value 的查找中。

- 键的模糊查找可以暂时使用时间复杂度为 *O(n)* 的遍历查找（Key的数量并不会很大，且不涉及数据具体内容的情况下，效率不会很低）
- 值的模糊查找则是开发难点了，它将会涉及大量数据的分析和查找，目前我正在构思相关的设计方案。