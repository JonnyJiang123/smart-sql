// SQL关键字解释字典
export const sqlKeywordExplanations: Record<string, string> = {
  // 查询相关
  'SELECT': '从数据库中检索数据。SELECT语句用于从一个或多个表中选择数据。',
  'FROM': '指定要查询的表名。FROM子句指定了数据的来源表。',
  'WHERE': '过滤查询结果。WHERE子句用于指定筛选条件，只返回满足条件的记录。',
  'ORDER BY': '对查询结果排序。ORDER BY子句用于按一个或多个列对结果集进行升序或降序排序。',
  'GROUP BY': '对结果进行分组。GROUP BY子句将具有相同值的行组合在一起，常与聚合函数一起使用。',
  'HAVING': '筛选分组后的结果。HAVING子句用于对GROUP BY分组后的结果进行过滤。',
  'LIMIT': '限制返回的记录数。LIMIT子句用于指定返回结果集的最大行数。',
  'OFFSET': '跳过指定数量的记录。OFFSET子句用于指定在返回结果之前要跳过的行数。',
  'DISTINCT': '去除重复记录。DISTINCT关键字用于返回唯一不同的值。',
  'AS': '为列或表指定别名。AS关键字用于给列、表或子查询设置一个临时名称。',
  
  // 连接相关
  'JOIN': '连接两个或多个表。JOIN用于根据相关列组合来自两个或多个表的行。',
  'INNER JOIN': '内连接，只返回两表中匹配的记录。INNER JOIN返回两个表中都有的匹配行。',
  'LEFT JOIN': '左连接，返回左表所有记录和右表匹配记录。LEFT JOIN返回左表的所有行，即使右表中没有匹配。',
  'RIGHT JOIN': '右连接，返回右表所有记录和左表匹配记录。RIGHT JOIN返回右表的所有行，即使左表中没有匹配。',
  'FULL JOIN': '全外连接，返回两表所有记录。FULL JOIN返回左表和右表中的所有行。',
  'CROSS JOIN': '交叉连接，返回两表的笛卡尔积。CROSS JOIN返回第一个表的每一行与第二个表的每一行的组合。',
  'ON': '指定连接条件。ON子句用于指定JOIN操作的连接条件。',
  'USING': '简化连接条件。USING子句用于在两表有相同列名时简化JOIN条件。',
  
  // 聚合函数
  'COUNT': '计算行数。COUNT()函数返回匹配指定条件的行数。',
  'SUM': '计算总和。SUM()函数返回数值列的总和。',
  'AVG': '计算平均值。AVG()函数返回数值列的平均值。',
  'MIN': '返回最小值。MIN()函数返回选定列的最小值。',
  'MAX': '返回最大值。MAX()函数返回选定列的最大值。',
  
  // 数据操作
  'INSERT': '向表中插入新记录。INSERT INTO语句用于向表中添加新数据行。',
  'UPDATE': '修改表中的现有记录。UPDATE语句用于更新表中的数据。',
  'DELETE': '从表中删除记录。DELETE语句用于删除表中的行。',
  'TRUNCATE': '删除表中所有数据。TRUNCATE TABLE语句用于快速清空表，比DELETE更快但无法回滚。',
  
  // 表操作
  'CREATE TABLE': '创建新表。CREATE TABLE语句用于在数据库中创建新表。',
  'ALTER TABLE': '修改表结构。ALTER TABLE语句用于添加、删除或修改表中的列。',
  'DROP TABLE': '删除表。DROP TABLE语句用于从数据库中删除整个表。',
  'RENAME': '重命名表或列。RENAME用于更改表或列的名称。',
  
  // 索引
  'INDEX': '创建索引以加快查询速度。索引可以显著提高数据检索速度。',
  'CREATE INDEX': '创建索引。CREATE INDEX语句用于在表上创建索引。',
  'DROP INDEX': '删除索引。DROP INDEX语句用于删除表上的索引。',
  
  // 约束
  'PRIMARY KEY': '主键约束。PRIMARY KEY用于唯一标识表中的每一行，不允许NULL值。',
  'FOREIGN KEY': '外键约束。FOREIGN KEY用于建立表之间的关系，确保引用完整性。',
  'UNIQUE': '唯一约束。UNIQUE约束确保列中的所有值都不同。',
  'NOT NULL': '非空约束。NOT NULL约束确保列不能有NULL值。',
  'CHECK': '检查约束。CHECK约束用于限制列中的值的范围。',
  'DEFAULT': '默认值约束。DEFAULT约束在未提供值时为列提供默认值。',
  
  // 集合操作
  'UNION': '合并多个SELECT结果并去重。UNION操作符用于合并两个或多个SELECT语句的结果集。',
  'UNION ALL': '合并多个SELECT结果保留重复。UNION ALL与UNION类似，但保留重复行。',
  'INTERSECT': '返回两个查询的交集。INTERSECT返回两个SELECT语句都包含的行。',
  'EXCEPT': '返回第一个查询独有的结果。EXCEPT返回第一个SELECT语句中有但第二个中没有的行。',
  
  // 子查询
  'IN': '检查值是否在列表中。IN操作符用于检查值是否匹配列表或子查询中的任何值。',
  'EXISTS': '检查子查询是否返回结果。EXISTS用于测试子查询是否返回任何行。',
  'ANY': '与子查询中的任意值比较。ANY操作符与比较运算符一起使用，表示满足任意一个条件即可。',
  'ALL': '与子查询中的所有值比较。ALL操作符表示必须满足所有条件。',
  
  // 逻辑操作
  'AND': '逻辑与运算符。AND用于组合多个条件，所有条件都必须为真。',
  'OR': '逻辑或运算符。OR用于组合多个条件，至少一个条件为真即可。',
  'NOT': '逻辑非运算符。NOT用于反转条件的真假值。',
  
  // 比较操作
  'LIKE': '模式匹配。LIKE用于在WHERE子句中搜索列中的指定模式。',
  'BETWEEN': '范围查询。BETWEEN用于选择在给定范围内的值。',
  'IS NULL': '检查空值。IS NULL用于检查列值是否为NULL。',
  'IS NOT NULL': '检查非空值。IS NOT NULL用于检查列值是否不为NULL。',
  
  // 字符串函数
  'CONCAT': '连接字符串。CONCAT()函数用于将两个或多个字符串连接在一起。',
  'SUBSTRING': '提取子字符串。SUBSTRING()函数用于从字符串中提取子字符串。',
  'UPPER': '转换为大写。UPPER()函数将字符串转换为大写字母。',
  'LOWER': '转换为小写。LOWER()函数将字符串转换为小写字母。',
  'LENGTH': '获取字符串长度。LENGTH()函数返回字符串的长度。',
  'TRIM': '删除空格。TRIM()函数删除字符串两端的空格。',
  
  // 日期函数
  'NOW': '获取当前日期时间。NOW()函数返回当前的日期和时间。',
  'CURDATE': '获取当前日期。CURDATE()函数返回当前日期。',
  'CURTIME': '获取当前时间。CURTIME()函数返回当前时间。',
  'DATE': '提取日期部分。DATE()函数从日期时间值中提取日期部分。',
  
  // 数学函数
  'ROUND': '四舍五入。ROUND()函数将数值四舍五入到指定的小数位数。',
  'CEIL': '向上取整。CEIL()函数返回大于或等于指定数值的最小整数。',
  'FLOOR': '向下取整。FLOOR()函数返回小于或等于指定数值的最大整数。',
  'ABS': '绝对值。ABS()函数返回数值的绝对值。',
  
  // 其他
  'CASE': '条件表达式。CASE表达式用于实现if-then-else逻辑。',
  'WHEN': 'CASE条件分支。WHEN用于指定CASE表达式中的条件。',
  'THEN': 'CASE结果值。THEN用于指定WHEN条件为真时返回的值。',
  'ELSE': 'CASE默认值。ELSE用于指定所有WHEN条件都不满足时返回的值。',
  'END': 'CASE结束标记。END用于标记CASE表达式的结束。',
  'VIEW': '视图。VIEW是基于SQL查询结果的虚拟表。',
  'TRANSACTION': '事务。TRANSACTION是一组作为单个逻辑工作单元执行的SQL语句。',
  'COMMIT': '提交事务。COMMIT用于永久保存事务的所有更改。',
  'ROLLBACK': '回滚事务。ROLLBACK用于撤销事务的所有更改。',
};

// 获取关键字解释
export function getKeywordExplanation(keyword: string): string | null {
  const upperKeyword = keyword.toUpperCase();
  return sqlKeywordExplanations[upperKeyword] || null;
}

// 检查是否是SQL关键字
export function isSqlKeyword(word: string): boolean {
  return word.toUpperCase() in sqlKeywordExplanations;
}
