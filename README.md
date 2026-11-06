# 智能知识点笔记与知识图谱联动桌面应用

一个本地运行的桌面知识管理工具：创建课程 → 拆分知识点 → 编写 Markdown 笔记 → 建立知识点关联 → 以知识图谱展示课程知识结构。

基于 **Tauri 2 + Vue 3 + TypeScript + SQLite** 构建，所有数据保存在本地，不依赖网络。

## 技术栈

| 层次 | 技术 |
| --- | --- |
| 桌面框架 | Tauri 2 |
| 前端 | Vue 3 `<script setup>` + TypeScript + Vite |
| 路由 / 状态 | Vue Router（hash 模式）+ Pinia |
| 后端 | Rust + rusqlite |
| 数据库 | SQLite（本地文件） |
| Markdown | markdown-it 渲染；导入导出使用系统文件对话框（`tauri-plugin-dialog`） |
| 知识图谱 | Vue Flow（阶段七接入） |

## 当前进度

- ✅ 开发环境、Tauri 项目初始化
- ✅ SQLite 表结构设计与数据库自动初始化
- ✅ 阶段一：项目基础架构（目录结构、Vue Router、Pinia、整体布局、左侧导航、路由切换）
- ✅ 阶段二：课程管理
  - 课程列表、课程详情、新增 / 编辑 / 删除完整 CRUD
  - 课程名称与描述校验、同名课程拦截、删除级联提示
- ✅ 阶段三：知识点管理
  - 多级知识点树、详情面板、新建 / 编辑 / 删除
  - 层级调整（降为子级 / 提升一级 / 修改父节点），删除时整棵子树级联
  - 知识点连同子树迁移到其它课程，课程详情与笔记工作区双向联通
- ✅ 阶段四：Markdown 笔记
  - 笔记库：列出全部笔记，按标题 / 正文搜索，打开到笔记工作区
  - Markdown 文件导入（预览标题与正文、选择目标知识点、覆盖提示）
  - 笔记导出为 `.md`（系统保存对话框，正文缺一级标题时自动补标题）
  - 笔记工作区：编辑 / 分屏 / 预览、800ms 自动保存、切换前落盘、删除二次确认
- ✅ 三档主题系统
  - 自动 / 日间 / 夜间主题，偏好持久化，自动模式跟随系统
- ⏳ 标签管理（阶段五）、知识点关联与关联建议（阶段六）、知识图谱（阶段七）等仍待后续阶段

完整任务清单见 [task.md](./task.md)。

## 快速开始

环境要求：Node.js、Rust stable、Visual Studio 2022 C++ 生成工具（Windows）。

```bash
npm install          # 安装前端依赖
npm run tauri dev    # 启动桌面应用（自动拉起 Vite）
npm run build        # 仅构建前端（vue-tsc 类型检查 + vite build）
npm run tauri build  # 打包桌面安装包
```

## 目录结构

```text
knowledge-graph-notes/
├─ src/                         前端
│  ├─ components/
│  │  ├─ layout/                AppLayout / AppHeader / AppSidebar / TitleBar / WindowResizeHandles
│  │  ├─ common/                AppIcon / PageHeader / PlaceholderPanel / BaseModal / ConfirmDialog / ThemeSwitcher
│  │  ├─ courses/               CourseFormDialog（课程管理页与笔记页共用）
│  │  ├─ knowledge/             知识点管理组件
│  │  │  ├─ KnowledgePointTree.vue         递归知识点树
│  │  │  ├─ KnowledgePointFormDialog.vue   新建 / 编辑知识点弹窗
│  │  │  └─ KnowledgePointDetailPanel.vue  知识点详情、层级调整与课程迁移
│  │  └─ notes/                 笔记工作区组件
│  │     ├─ NotesSidebar.vue        课程与知识点目录（新建入口、拖拽宽度、笔记库入口）
│  │     ├─ KnowledgeTreeNode.vue   递归知识点树
│  │     ├─ NoteWorkspace.vue       标题、编辑 / 分屏 / 预览、保存状态、导入导出
│  │     ├─ MarkdownImportDialog.vue Markdown 导入弹窗（预览、目标知识点、覆盖提示）
│  │     ├─ NoteMetaPanel.vue       所属知识点、标签、相关知识点
│  │     ├─ SaveStatusIndicator.vue 保存状态
│  │     └─ KnowledgePointFormDialog.vue 新建知识点弹窗
│  ├─ views/                    首页、课程、课程详情、知识点管理、笔记、笔记库、图谱、设置、404
│  ├─ router/                   路由表（meta 驱动导航 / 沉浸式页面）
│  ├─ stores/
│  │  ├─ app.ts                 应用级界面状态（侧边栏折叠）
│  │  ├─ theme.ts               主题状态（唯一管理入口）
│  │  ├─ courses.ts             课程列表与增删改查
│  │  ├─ knowledgePoints.ts     知识点树、详情与增删改查
│  │  ├─ notes.ts               笔记数据、草稿、自动保存
│  │  └─ workspace.ts           笔记页布局偏好（宽度、面板、视图模式）
│  ├─ services/
│  │  ├─ tauri.ts               invoke 统一封装与错误类型
│  │  ├─ courses.ts             课程 CRUD API
│  │  ├─ knowledgePoints.ts     知识点树 / 详情 / 增删改查 / 层级调整 API
│  │  ├─ notes.ts               知识点 / 笔记 / 标签 / 关联 / 搜索 / 导入导出 API
│  │  ├─ markdownFiles.ts       系统文件选择框与 Markdown 导出封装
│  │  └─ theme.ts               主题读取、解析、持久化
│  ├─ utils/                    Markdown 渲染、时间与字数格式化
│  ├─ types/                    领域模型、RouteMeta 扩展
│  ├─ styles/                   设计令牌、基础样式、Markdown 样式
│  ├─ config.ts                 应用名称、版本
│  ├─ App.vue
│  └─ main.ts
├─ src-tauri/                   后端
│  ├─ src/
│  │  ├─ commands/              命令层：course / knowledge_point / note / tag / relation
│  │  ├─ models/                数据模型：与表结构一一对应
│  │  ├─ db/mod.rs              SQLite 连接与初始化
│  │  ├─ lib.rs                 应用入口与命令注册
│  │  └─ main.rs
│  ├─ migrations/001_initial.sql
│  ├─ capabilities/default.json
│  └─ tauri.conf.json
├─ public/
├─ index.html
├─ vite.config.ts
└─ package.json
```

前端统一使用 `@` 别名指向 `src`。

## 窗口与标题栏

应用使用无边框窗口（`tauri.conf.json` 中 `decorations: false`），由自定义标题栏承担系统标题栏职责：

- 标题栏左侧显示应用标识与名称；空白区域可拖动窗口，双击可最大化 / 还原；
- 右侧提供最小化、最大化 / 还原、关闭按钮，关闭前会先保存笔记草稿，避免丢失最后几秒的输入；
- 通过 `WindowResizeHandles` 在窗口四边与四角提供原生缩放（`startResizeDragging`），弥补无边框窗口丢失的默认缩放边框；
- 标题栏、按钮悬停态、关闭按钮的红色反馈全部使用主题变量，跟随日间 / 夜间 / 自动主题同步变化；
- 窗口使用 `shadow: false`，避免无边框窗口四周出现透出桌面的透明边框。

相关文件：`src/components/layout/TitleBar.vue`、`src/components/layout/WindowResizeHandles.vue`；
窗口权限集中在 `src-tauri/capabilities/default.json`。

## 主题系统

三种模式：**自动 / 日间 / 夜间**，入口只在「设置 → 外观」，其它页面不显示切换控件。

- 状态集中在 [`src/stores/theme.ts`](src/stores/theme.ts)，所有组件只读取这个 store，不各自处理主题；
- 颜色定义在 [`src/styles/tokens.css`](src/styles/tokens.css) 的 `:root`（日间）与 `:root[data-theme='dark']`（夜间）；
- 自动模式通过 `matchMedia('(prefers-color-scheme: dark)')` 解析最终主题，并监听 `change` 事件实时响应系统变化；
- 主题偏好保存在 `localStorage` 的 `kgn:theme-mode`（取值 `auto | light | dark`）；
- `index.html` 在首屏渲染前根据偏好设置 `data-theme`，避免深色模式启动时白屏闪烁；
- 全局只有 `<html data-theme>` 一个开关，笔记页、设置页、侧边栏、弹窗等所有界面统一生效。

### 手动验证主题

1. 进入「设置 → 外观」，切换自动 / 日间 / 夜间，界面立即变化，按钮高亮同步；
2. 首页、课程、笔记等其它页面不再出现主题切换入口；
3. 选择「自动」后，在 Windows 设置 → 个性化 → 颜色 中切换浅色 / 深色，应用会跟随变化；
4. 重启应用后仍保留上次选择（`localStorage` 持久化）。

## 课程管理

`/courses` 提供课程的完整增删改查，所有数据直接写入本地 SQLite：

- 列表展示课程名称、描述、知识点 / 笔记数量与最近更新时间，并提供详情、编辑、删除入口；
- 新建与编辑复用同一个弹窗；课程名称必填（≤60 字）、描述可选（≤500 字），后端会再次校验并拒绝同名课程；
- 删除前二次确认，并明确提示课程下的知识点、笔记、标签关联与知识点关联会级联删除；
- 详情页展示课程简介、知识点结构（可跳转到知识点管理并选中对应知识点）以及笔记工作区 / 知识图谱入口，可直接编辑或删除课程；
- 课程变更后进入笔记工作区会强制刷新目录；若当前选中的知识点已被删除，会自动清理失效状态，避免草稿写回已删除的数据。

## 知识点管理

`/knowledge-points` 按课程维护多级知识点目录，也可以从课程详情页带上课程 / 知识点参数直接进入：

- 左栏切换课程并显示每门课程的知识点数；中栏显示当前课程的知识点树（层级缩进、子项数与笔记数）；
- 顶部或行内 `+` 新建知识点，可指定父知识点；行内可直接编辑 / 删除；
- 右侧详情面板展示面包屑路径、层级、子知识点 / 后代 / 笔记统计、描述与创建 / 更新时间；
- 层级调整：「降为子级」成为同级上一个知识点的子知识点，「提升一级」与父知识点同级；也可以在编辑弹窗中直接修改父节点；
- 删除知识点会连同全部后代知识点及其笔记、相关关联一起删除，确认框会给出准确数量；
- 迁移课程会把知识点连同子树移动到另一门课程，根节点自动成为目标课程的顶层知识点；
- 「在笔记中打开」跳转到笔记工作区并选中该知识点，方便继续编写笔记。

## 页面切换

- 首屏空闲时预取所有懒加载页面模块，点击导航后不再等待 JS 分块下载；
- 页面切换为 90ms 淡出 + 150ms 淡入，避免长时间空白；
- 笔记页的沉浸式布局（隐藏应用侧边栏、去掉内容区留白）在旧页面离场后一次性切换，旧页面不会在淡出时突然收缩；
- 内容区滚动位置在新页面进入时归零，旧页面保持原位置平滑退出。

## 笔记工作区

`/notes` 是一个沉浸式三栏页面，进入时应用级导航会自动收起为图标栏，把宽度留给编辑区。

### 左栏：课程与知识点

- 课程默认展开，点击课程行收起 / 展开；知识点支持多级展开 / 收起；
- 当前知识点使用浅色高亮 + 左侧色条标记；
- 顶部 `+` 新建课程，课程行悬浮 `+` 或“添加”新建知识点，支持指定父知识点；
- 右边缘可拖动调整宽度（200–380px），顶部工具栏按钮可整体折叠。

### 中栏：笔记编辑

- 顶部显示课程 / 知识点路径和笔记标题；
- 编辑 / 分屏 / 预览三种模式，分屏在窄容器下自动改为上下排列；
- 800ms 防抖自动保存，`Ctrl+S` 立即保存；
- 保存状态：未保存 / 保存中 / 已保存 / 保存失败（可重试）；
- 切换知识点前会先保存当前草稿，避免内容丢失；
- 删除笔记需要二次确认；
- 笔记内容为空且标题未修改时不会创建空笔记；
- 工具栏提供「导入 Markdown」（导入到任意知识点，可先预览）与「导出 Markdown」（先落盘草稿再弹出系统保存框）。

Markdown 由已安装的 markdown-it 渲染（禁用内联 HTML），标题、列表、引用、代码块、表格等样式见 [`src/styles/markdown.css`](src/styles/markdown.css)，全部使用主题变量。

### 右栏：辅助信息

- 所属知识点与课程、更新时间、字数；
- 笔记标签：输入名称添加，已有标签可通过 datalist 快速选择，点击 × 移除；
- 相关知识点：读取 `knowledge_relations`，点击可跳转到对应知识点；
- 关联建议：阶段六接入，目前为说明性占位；
- 顶部 × 或工具栏按钮可隐藏面板，把空间让给编辑区。

## 笔记库

`/note-library` 是笔记视角的集中入口，与按知识点组织的笔记工作区互补：

- 列出全部笔记，展示标题、正文摘要、所属课程 / 知识点、字数与更新时间；
- 顶部搜索框按标题与正文检索（300ms 防抖，标题命中的结果排在前面），支持一键清空；
- 点击笔记或「打开」跳转到笔记工作区并选中对应知识点；
- 「导出」弹出系统保存框，把笔记写成 `.md` 文件（正文缺少一级标题时自动补上笔记标题）；
- 「导入 Markdown」选择本地文件后，可预览正文、修改标题、选择目标知识点；目标知识点已有笔记时会提示将被覆盖；
- 导入成功后直接进入笔记工作区，导出 / 导入结果通过顶部提示条反馈。

导入解析规则：文件首行的 `# 一级标题` 会作为笔记标题并从正文移除；没有一级标题时使用文件名；仅支持 UTF-8 文本，文件上限 2 MB。

## 后端命令

为支撑课程管理与笔记工作区的真实数据读写，当前注册的命令如下（**未修改数据库结构**，数据库仍使用 `001_initial.sql`）：

| 模块 | 命令 |
| --- | --- |
| 课程 | `list_courses` / `get_course` / `create_course` / `update_course` / `delete_course` |
| 知识点 | `list_knowledge_points` / `list_knowledge_points_by_course` / `get_knowledge_point_tree` / `get_knowledge_point_detail` / `create_knowledge_point` / `update_knowledge_point` / `move_knowledge_point` / `move_knowledge_point_to_course` / `delete_knowledge_point` |
| 笔记 | `get_note` / `save_note` / `delete_note` / `list_notes` / `search_notes` / `export_note_markdown` / `read_markdown_file` |
| 标签 | `list_tags` / `create_tag` / `list_note_tags` / `attach_note_tag` / `detach_note_tag` |
| 关联 | `list_relations` |

课程、知识点与笔记命令已具备完整 CRUD、后端校验与文件读写；标签筛选与知识点关联属于后续阶段。

> Markdown 导入 / 导出使用 `tauri-plugin-dialog` 调用系统文件选择框，权限为 `src-tauri/capabilities/default.json` 中的 `dialog:default`；文件读写本身由 Rust 命令完成，不依赖前端文件系统插件。

## 数据库

- 位置：Tauri `app_data_dir()` 下的 `knowledge_graph.db`（Windows 一般为 `%APPDATA%\<应用标识>\knowledge_graph.db`）。
- 启动时由 `db::initialize_database()` 执行 `migrations/001_initial.sql` 建表，并开启外键约束。
- 时间字段统一以 ISO 8601 字符串存储；前端 `types` 中的字段名与表结构、Rust serde 输出保持一致（snake_case）。

## 开发约定

1. 每次只实现一个完整功能模块，优先 Rust 后端，再连接 Vue 前端。
2. 每个模块完成后运行 `npm run build` 与 `cargo check`。
3. 不随意覆盖 Tauri 脚手架原有文件；数据库结构变更使用新的迁移文件。
4. 界面只使用 `src/styles/tokens.css` 中的主题变量，不写死颜色。
5. 不为界面引入模拟数据，所有展示内容来自 SQLite。
6. 每完成一个阶段，更新 `task.md`。
