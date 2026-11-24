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
| 知识图谱 | Vue Flow（层次布局 / 力导向布局） |

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
- ✅ 阶段五：标签管理
  - 标签管理页：搜索标签、新建 / 重命名 / 删除、查看每个标签下的笔记
  - 笔记标签：编辑器右侧面板添加 / 移除标签，标签随笔记展示
  - 按标签筛选：笔记库标签筛选条与 `?tag=` 深链，筛选可与关键字搜索组合
- ✅ 阶段六：知识点关联
  - 关联类型字典：前置 / 相关 / 包含 / 对比 / 延伸，可增改自定义类型与颜色
  - 关联管理：手动创建、编辑类型与描述、按课程 / 知识点 / 类型查询、删除
  - 一致性校验：阻止自关联、同类型重复关联与无向类型的反向重复
  - 关联建议：融合标题、描述与笔记文本的本地相似度，展示得分与共同关键词理由，支持接受 / 拒绝
  - 联动：笔记工作区侧栏展示相关知识点与建议，知识点 / 课程详情提供关联入口
- ✅ 阶段七：知识图谱
  - Vue Flow 画布：知识点节点、层级虚线、按类型着色的语义关联连线（有向关系带箭头）
  - 交互：节点拖拽、滚轮缩放、空白拖动平移、点击节点查看详情、自动适配视图
  - 筛选：按课程、层级上限、关联类型筛选，层级连线 / 关系标签开关
  - 布局：层次树布局与确定性力导向布局切换，重新布局与数据刷新
  - 联动：详情面板可跳转笔记、知识点管理与知识关联页
- ✅ 三档主题系统
  - 自动 / 日间 / 夜间主题，偏好持久化，自动模式跟随系统
- ✅ 阶段八：首页与辅助功能
  - 首页统计课程 / 知识点 / 笔记 / 标签 / 关联数量，展示最近编辑的笔记与最近创建的知识点
  - 全局搜索面板（顶栏入口 / Ctrl + K）：课程、知识点、笔记、标签与知识点关联分组结果，支持键盘选择与页面跳转
  - 设置页：数据库路径、大小、记录数与 SQLite 版本，数据库备份 / 恢复（恢复前自动生成安全副本），以及关于信息
- ✅ 阶段九：测试与优化
  - 后端 73 项、前端 21 项自动化测试，`cargo test` / `npm run test:unit` / `npm run build` 全部通过
  - 数据库持久化与异常数据测试、弹窗 Esc 交互修复、vendor 分包、数据安全性审计
- ✅ 阶段十：应用打包与发布
  - 产品名「智能知识点笔记」，可执行文件 `knowledge-graph-notes.exe`，版本 1.0.0
  - 全套应用图标（SVG 源图 → ICO / ICNS / PNG / Windows Store 徽标）与前端 favicon
  - NSIS（当前用户安装，无需管理员）与 MSI（需要管理员权限）安装包
  - 安装包安装、安装后启动、数据库持久化验证通过，详见[阶段十测试报告](./docs/阶段十测试报告.md)
  - 面向用户与发布流程的文档：[用户手册](./docs/用户手册.md)、[v1.0.0 发布说明](./docs/发布说明-v1.0.0.md)

完整任务清单见 [task.md](./task.md)。

## 本地开发

环境要求：Node.js 20+、Rust stable、Visual Studio 2022 C++ 生成工具（Windows）。

```bash
npm install                    # 安装前端依赖
npm run tauri dev              # 启动桌面应用（自动拉起 Vite）
npm run build                  # 前端类型检查 + 生产构建
npm run test:unit              # 前端单元测试（Vitest）
cargo test --manifest-path src-tauri/Cargo.toml   # 后端测试
npm run tauri build            # 打包 NSIS + MSI 安装包
```

## 安装与发布

v1.0.0 安装包位于 `src-tauri/target/release/bundle/`：

| 安装包 | 文件 | 说明 |
| --- | --- | --- |
| NSIS（推荐） | `nsis/智能知识点笔记_1.0.0_x64-setup.exe` | 当前用户安装，默认写入 `%LOCALAPPDATA%\智能知识点笔记`，无需管理员权限 |
| MSI | `msi/智能知识点笔记_1.0.0_x64_zh-CN.msi` | 面向域 / 企业分发（`ALLUSERS=1`），安装需要管理员权限 |

- 系统要求：Windows 10 1809+ / Windows 11 x64；安装包内置 WebView2 引导安装程序，缺少运行时会在安装阶段自动补齐。
- 安装包创建开始菜单与桌面快捷方式；卸载入口为「Windows 设置 → 应用 → 智能知识点笔记」，也可运行安装目录下的 `uninstall.exe`。
- 用户数据与安装目录分离：数据库位于 `%APPDATA%\com.gaotiankai.knowledge-graph-notes\knowledge_graph.db`，卸载不会删除数据，重装后仍可继续使用。
- 发布前请阅读[v1.0.0 发布说明](./docs/发布说明-v1.0.0.md)；安装、启动与数据持久化的实测结果见[阶段十测试报告](./docs/阶段十测试报告.md)，日常使用见[用户手册](./docs/用户手册.md)。

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
│  │  ├─ tags/                  标签管理组件
│  │  │  └─ TagFormDialog.vue            新建 / 重命名标签弹窗
│  │  ├─ relations/             知识点关联组件
│  │  │  ├─ RelationFormDialog.vue       新建 / 编辑关联弹窗
│  │  │  └─ RelationTypeDialog.vue       关联类型管理弹窗
│  │  ├─ graph/                 知识图谱组件
│  │  │  ├─ GraphCanvas.vue             Vue Flow 画布、缩放平移、布局与图例
│  │  │  ├─ GraphNode.vue               知识点节点
│  │  │  └─ GraphDetailPanel.vue        节点详情与联动入口
│  │  ├─ search/                全局搜索
│  │  │  └─ GlobalSearchPalette.vue     Ctrl + K 搜索面板、分组结果与键盘导航
│  │  └─ notes/                 笔记工作区组件
│  │     ├─ NotesSidebar.vue        课程与知识点目录（新建入口、拖拽宽度、笔记库入口）
│  │     ├─ KnowledgeTreeNode.vue   递归知识点树
│  │     ├─ NoteWorkspace.vue       标题、编辑 / 分屏 / 预览、保存状态、导入导出
│  │     ├─ MarkdownImportDialog.vue Markdown 导入弹窗（预览、目标知识点、覆盖提示）
│  │     ├─ NoteMetaPanel.vue       所属知识点、标签、相关知识点与关联建议
│  │     ├─ SaveStatusIndicator.vue 保存状态
│  │     └─ KnowledgePointFormDialog.vue 新建知识点弹窗
│  ├─ views/                    首页、课程、课程详情、知识点管理、笔记、笔记库、标签管理、知识关联、图谱、设置、404
│  ├─ router/                   路由表（meta 驱动导航 / 沉浸式页面）
│  ├─ stores/
│  │  ├─ app.ts                 应用级界面状态（侧边栏折叠、全局搜索开关）
│  │  ├─ theme.ts               主题状态（唯一管理入口）
│  │  ├─ courses.ts             课程列表与增删改查
│  │  ├─ knowledgePoints.ts     知识点树、详情与增删改查
│  │  ├─ notes.ts               笔记数据、草稿、自动保存与关联建议
│  │  ├─ tags.ts                标签列表、搜索与标签笔记
│  │  ├─ relations.ts           关联列表、类型、建议与增删改查
│  │  ├─ graph.ts               图谱课程树、筛选、布局与选中状态
│  │  └─ workspace.ts           笔记页布局偏好（宽度、面板、视图模式）
│  ├─ services/
│  │  ├─ tauri.ts               invoke 统一封装与错误类型
│  │  ├─ courses.ts             课程 CRUD API
│  │  ├─ knowledgePoints.ts     知识点树 / 详情 / 增删改查 / 层级调整 API
│  │  ├─ notes.ts               知识点 / 笔记 / 搜索 / 导入导出 API
│  │  ├─ tags.ts                标签增删改查与笔记标签关联 API
│  │  ├─ relations.ts           关联、关联类型与关联建议 API
│  │  ├─ dashboard.ts           首页统计、最近笔记与最近知识点 API
│  │  ├─ search.ts              全局搜索 API
│  │  ├─ database.ts            数据库信息、备份 / 恢复与系统文件对话框
│  │  ├─ markdownFiles.ts       系统文件选择框与 Markdown 导出封装
│  │  └─ theme.ts               主题读取、解析、持久化
│  ├─ utils/                    Markdown 渲染、时间 / 字数 / 文件大小格式化、图谱布局算法
│  ├─ types/                    领域模型、RouteMeta 扩展
│  ├─ styles/                   设计令牌、基础样式、Markdown 样式
│  ├─ config.ts                 应用名称、版本
│  ├─ App.vue
│  └─ main.ts
├─ src-tauri/                   后端
│  ├─ src/
│  │  ├─ commands/              命令层：course / knowledge_point / note / tag / relation / dashboard / search / backup
│  │  ├─ models/                数据模型：与表结构一一对应
│  │  ├─ db/mod.rs              SQLite 连接、迁移执行与表数据量统计
│  │  ├─ text.rs                摘要、字符统计与搜索片段
│  │  ├─ point_path.rs          知识点层级路径
│  │  ├─ lib.rs                 应用入口与命令注册
│  │  └─ main.rs
│  ├─ migrations/001_initial.sql
│  ├─ migrations/002_relations.sql
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
- 窗口使用 `shadow: true`，无边框窗口四周保留系统阴影，与桌面形成层次；`decorations: false` 时由自定义标题栏负责拖动与窗口按钮。

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

## 首页

`/` 是知识库的概览页，数据全部来自后端 `get_dashboard_overview` 的一次聚合查询：

- 顶部统计课程、知识点、笔记、标签与知识点关联数量，点击可进入对应模块；
- 「最近编辑的笔记」按 `updated_at` 倒序展示 5 条，带正文摘要、所属课程 / 知识点、更新时间与字数，点击直接打开笔记工作区；
- 「最近创建的知识点」按 `created_at` 倒序展示 5 条，带所属课程与父级路径、创建时间与笔记数量，点击进入知识点管理并选中该知识点；
- 右侧提供全局搜索入口、快速访问与数据备份提示；加载中显示骨架屏，读取失败时保留错误提示与重试按钮。

## 全局搜索

顶栏搜索按钮与 `Ctrl + K`（macOS 为 `Cmd + K`）可在任意页面唤起全局搜索面板：

- 输入关键字后 220ms 防抖调用 `global_search`，同时检索课程名称 / 描述、知识点标题 / 描述、笔记标题 / 正文、标签名称与知识点关联（描述、两端标题、类型）；
- 结果按「课程 / 知识点 / 笔记 / 标签 / 知识关联」分组，展示所属课程与层级路径、笔记正文命中片段、标签使用数量与关联类型；
- 支持 `↑` / `↓` 在全部结果间连续移动、`Enter` 打开、`Esc` 关闭；点击结果跳转到课程详情、知识点管理、笔记工作区、笔记库标签筛选或知识关联页；
- 关键字中的 `%` / `_` 按字面量处理；空关键字不会发起查询，输入过程使用请求序号丢弃过期结果。

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
- 笔记页的沉浸式布局（去掉内容区留白）在旧页面离场后一次性切换，旧页面不会在淡出时突然收缩；
- 应用侧边栏只响应用户手动折叠，切换笔记 / 知识图谱等页面不会自动收起；
- 内容区滚动位置在新页面进入时归零，旧页面保持原位置平滑退出。

## 笔记工作区

`/notes` 是一个沉浸式三栏页面，进入时只去掉内容区留白，应用级导航保持用户手动设置的状态。

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
- 相关知识点：读取 `knowledge_relations`，显示关联类型与颜色，点击可跳转到对应知识点；
- 关联建议：展示后端根据名称、描述与笔记内容生成的候选、相似度与推荐理由，可直接接受或忽略；
- 顶部 × 或工具栏按钮可隐藏面板，把空间让给编辑区。

## 笔记库

`/note-library` 是笔记视角的集中入口，与按知识点组织的笔记工作区互补：

- 列出全部笔记，展示标题、正文摘要、所属课程 / 知识点、标签、字数与更新时间；
- 顶部搜索框按标题与正文检索（300ms 防抖，标题命中的结果排在前面），支持一键清空；
- 标签筛选条按标签过滤笔记，可与关键字搜索组合使用；筛选条件写入路由（`/note-library?tag=<id>`），可从标签管理页直接带标签进入；
- 点击笔记或「打开」跳转到笔记工作区并选中对应知识点；
- 「导出」弹出系统保存框，把笔记写成 `.md` 文件（正文缺少一级标题时自动补上笔记标题）；
- 「导入 Markdown」选择本地文件后，可预览正文、修改标题、选择目标知识点；目标知识点已有笔记时会提示将被覆盖；
- 导入成功后直接进入笔记工作区，导出 / 导入结果通过顶部提示条反馈。

导入解析规则：文件首行的 `# 一级标题` 会作为笔记标题并从正文移除；没有一级标题时使用文件名；仅支持 UTF-8 文本，文件上限 2 MB。

## 标签管理

`/tags` 用标签给笔记分类，并在标签视角下检索笔记：

- 左侧标签列表显示每个标签的关联笔记数量，支持按名称搜索（250ms 防抖）；
- 新建标签（同名标签会复用已有记录）、重命名标签（同名会拦截）、删除标签（二次确认，提示会影响多少条笔记，笔记本身不会被删除）；
- 右侧展示选中标签下的笔记，点击可直接打开到笔记工作区；底部可跳转笔记库并带上标签筛选；
- 笔记工作区的右侧辅助面板可以给当前笔记添加 / 移除标签，并跳转到标签管理页。

## 知识点关联

`/relations` 用于建立和浏览知识点之间的语义关联，与父子层级相互独立：

- 左侧按课程筛选知识点并显示每个知识点的关联数量，支持按名称 / 描述搜索；
- 中间展示当前知识点的关联：类型标签（含颜色与有向 / 无向）、两端知识点、描述、课程路径，可编辑类型与描述、删除关联；
- 顶部「新建关联」可跨课程选择源 / 目标知识点与关联类型；后端阻止自关联、同类型重复关联，以及无向类型的反向重复；
- 「关联类型」弹窗维护类型字典：内置前置、相关、包含、对比、延伸，可改名 / 换色 / 调整有向性，或新增自定义类型；内置与被使用的类型不允许删除；
- 右侧关联建议只比较同一课程内的知识点，已有正式关联、已接受或已拒绝的知识点对会被排除；接受后写入正式关联，拒绝后不再重复推荐；
- 知识点详情面板展示语义关联数量并提供入口，笔记工作区侧栏也会展示当前知识点的相关知识点与建议。

关联建议完全本地计算，不依赖网络或大模型：名称、描述、笔记正文分别做中文二元字组 / 英文单词切分，按 `0.45·名称余弦 + 0.25·描述余弦 + 0.20·笔记余弦 + 0.10·Jaccard` 融合得分，超过阈值后取 Top-K，并把共同关键词整理成推荐理由。每个知识点的笔记文本截断到 3000 字，建议结果以无序知识点对唯一保存。

## 知识图谱

`/graph` 是沉浸式图谱页，把课程知识点与语义关联放在同一张可交互画布中：

- 节点来自当前课程的知识点树，展示标题、层级、子项数、笔记数与关联数量；
- 父子层级生成灰色虚线边；语义关联生成按类型着色的实线边，有向类型（前置 / 包含 / 延伸）带箭头；
- 交互：拖动节点调整位置、滚轮缩放、拖拽空白平移、点击节点查看详情，右下角提供缩放与适配视图按钮，右上角「重新布局」恢复自动布局；
- 筛选：课程切换、层级上限（全部 / 仅顶层 / 前 N 层）、关联类型多选、层级连线开关、关系标签开关；
- 布局：层次树布局把层级放在横轴、同级兄弟放在纵轴，并按子树高度排布；关系网络使用确定性力导向模拟，层级边与语义边共同作为弹簧，节点位置可复现；
- 详情面板展示知识点路径、子项 / 笔记 / 关联统计、描述与关联列表，可跳转到笔记工作区、知识点管理或知识关联页；
- 数据刷新从 SQLite 重新读取知识点树、关联与类型，保留当前课程、筛选与选中节点。

图谱只绘制所选课程内部两端都可见的关联；跨课程关联仍在知识关联页管理。布局算法位于 [`src/utils/graphLayout.ts`](src/utils/graphLayout.ts)，不依赖额外布局库，节点规模增大时可在阶段九替换为增量布局或 Web Worker。

## 设置与数据备份

`/settings` 集中管理外观、数据库与搜索入口：

- 「外观」保留自动 / 日间 / 夜间主题切换与当前生效配色说明；
- 「数据与备份」显示数据库文件路径、文件大小、最后修改时间、SQLite 版本与各表记录数，并提供「备份数据库」「恢复数据库」「在文件夹中显示」；
- 备份：系统保存对话框选择位置后，Rust 端用 `VACUUM INTO` 写入同目录临时文件，校验通过后再替换目标文件（目标已存在时覆盖），返回实际路径与大小；
- 恢复：系统打开对话框选择 `.db / .sqlite / .sqlite3` 文件，Rust 端依次校验 SQLite 文件头、`PRAGMA quick_check` 与六张业务表是否存在；通过后先在数据库同目录生成 `knowledge_graph-before-restore-<时间>.db` 安全副本，再替换当前数据库并重放幂等迁移，最后返回恢复后的数据库信息与安全副本路径；
- 恢复成功后界面提示结果并自动重新加载应用，确保所有页面读取恢复后的数据；恢复失败会尽量回滚到恢复前的安全副本；
- 「全局搜索」卡片提供 Ctrl + K 快捷键说明与打开入口；「关于」展示应用名称、版本、开发阶段、技术栈与数据存储方式。

## 后端命令

为支撑各模块的真实数据读写，当前注册的命令如下（数据库结构由 `001_initial.sql` + `002_relations.sql` 两个迁移维护）：

| 模块 | 命令 |
| --- | --- |
| 课程 | `list_courses` / `get_course` / `create_course` / `update_course` / `delete_course` |
| 知识点 | `list_knowledge_points` / `list_knowledge_points_by_course` / `get_knowledge_point_tree` / `get_knowledge_point_detail` / `create_knowledge_point` / `update_knowledge_point` / `move_knowledge_point` / `move_knowledge_point_to_course` / `delete_knowledge_point` |
| 笔记 | `get_note` / `save_note` / `delete_note` / `list_notes` / `search_notes` / `export_note_markdown` / `read_markdown_file` |
| 标签 | `list_tags` / `list_tag_overview` / `search_tags` / `create_tag` / `update_tag` / `delete_tag` / `list_note_tags` / `attach_note_tag` / `detach_note_tag` |
| 关联 | `list_relations` / `list_relation_details` / `list_relation_details_for_point` / `list_relation_details_by_course` / `create_relation` / `update_relation` / `delete_relation` |
| 关联类型 | `list_relation_types` / `create_relation_type` / `update_relation_type` / `delete_relation_type` |
| 关联建议 | `generate_relation_suggestions` / `list_relation_suggestions` / `accept_relation_suggestion` / `reject_relation_suggestion` |
| 首页 | `get_dashboard_overview` |
| 全局搜索 | `global_search` |
| 数据库 | `get_database_info` / `backup_database` / `restore_database` |

课程、知识点、笔记、标签与关联命令均具备完整 CRUD、后端校验与所需的事务 / 约束处理。知识图谱复用 `get_knowledge_point_tree`、`list_relation_details_by_course` 与 `list_relation_types`；首页与全局搜索复用现有业务表，通过聚合查询与共享的文本 / 层级路径工具实现；数据库备份与恢复使用 rusqlite 连接与标准文件操作完成。阶段八未新增数据库迁移。

> Markdown 导入 / 导出使用 `tauri-plugin-dialog` 调用系统文件选择框，权限为 `src-tauri/capabilities/default.json` 中的 `dialog:default`；文件读写本身由 Rust 命令完成，不依赖前端文件系统插件。

## 数据库

- 位置：Tauri `app_data_dir()` 下的 `knowledge_graph.db`（Windows 一般为 `%APPDATA%\<应用标识>\knowledge_graph.db`）。
- 启动时由 `db::initialize_database()` 按顺序执行 `migrations/001_initial.sql` 与 `migrations/002_relations.sql`，并开启外键约束；迁移均使用 `IF NOT EXISTS` / `INSERT OR IGNORE`，可安全重放。
- `knowledge_relations` 通过 `CHECK (source_id <> target_id)` 与 `UNIQUE (source_id, target_id, relation_type)` 在数据库层阻止自关联和重复关联；`relation_types` 维护类型名称、颜色、是否有向；`relation_suggestions` 以无序知识点对唯一保存建议状态（pending / accepted / rejected）。
- 设置页的备份使用 SQLite `VACUUM INTO` 生成一致的数据库副本；恢复前会在数据库同目录保留 `knowledge_graph-before-restore-*.db` 安全副本，恢复文件名与路径由用户通过系统对话框选择。
- 时间字段统一以 ISO 8601 字符串存储；前端 `types` 中的字段名与表结构、Rust serde 输出保持一致（snake_case）。

## 开发约定

1. 每次只实现一个完整功能模块，优先 Rust 后端，再连接 Vue 前端。
2. 每个模块完成后运行 `npm run build` 与 `cargo check`。
3. 不随意覆盖 Tauri 脚手架原有文件；数据库结构变更使用新的迁移文件。
4. 界面只使用 `src/styles/tokens.css` 中的主题变量，不写死颜色。
5. 不为界面引入模拟数据，所有展示内容来自 SQLite。
6. 每完成一个阶段，更新 `task.md`。
