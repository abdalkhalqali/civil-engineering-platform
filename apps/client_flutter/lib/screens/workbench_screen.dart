import 'package:flutter/material.dart';

import '../ffi_bridge/generated/api.dart';
import '../widgets/engineering_viewport.dart';

enum ModelingTool { select, column, beam, slab, wall, foundation }

class WorkbenchScreen extends StatefulWidget {
  const WorkbenchScreen({required this.kernelReady, super.key});

  final bool kernelReady;

  @override
  State<WorkbenchScreen> createState() => _WorkbenchScreenState();
}

class _WorkbenchScreenState extends State<WorkbenchScreen> {
  ViewportInteraction _interaction = ViewportInteraction.orbit;
  ViewportProjection _projection = ViewportProjection.perspective;
  ModelingTool _activeTool = ModelingTool.select;
  String? _selectedElement;
  String _projectName = 'مشروع تجريبي';
  String _projectType = 'مبنى إنشائي';
  String _displayUnit = 'm';
  double _landArea = 1200;
  WorkspaceSnapshot? _snapshot;
  final Set<String> _hiddenCategories = <String>{};
  bool _showGrid = true;
  bool _projectDirty = false;
  int _viewResetToken = 0;

  static const _projectTypes = [
    'مبنى إنشائي',
    'جسر',
    'أعمال ترابية وتهيئة أرض',
  ];

  @override
  void initState() {
    super.initState();
    if (widget.kernelReady) {
      try {
        _snapshot = createWorkspaceSnapshot(
          name: _projectName,
          projectType: _projectType,
          landAreaM2: _landArea,
        );
      } catch (error) {
        debugPrint('Initial workspace snapshot failed: $error');
      }
    }
  }

  void _checkKernel() {
    try {
      getKernelStatus();
      _showMessage('تم الاتصال بالنواة الهندسية بنجاح.');
    } catch (error) {
      _showMessage('تعذر الاتصال بالنواة: $error', error: true);
    }
  }

  void _showMessage(String message, {bool error = false}) {
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(
        SnackBar(
          content: Text(message),
          backgroundColor: error ? const Color(0xffb3261e) : null,
          behavior: SnackBarBehavior.floating,
        ),
      );
  }

  ElementSnapshot? get _selectedSnapshot {
    final selected = _selectedElement;
    if (selected == null) return null;
    for (final element in _snapshot?.elements ?? const <ElementSnapshot>[]) {
      if (element.name == selected) return element;
    }
    return null;
  }

  void _selectTool(ModelingTool tool) {
    setState(() => _activeTool = tool);
    if (tool != ModelingTool.select) {
      _showMessage(
        '${_toolLabel(tool)}: اختر نقطة في المشهد لبدء الإدخال الهندسي.',
      );
    }
  }

  String _toolLabel(ModelingTool tool) {
    switch (tool) {
      case ModelingTool.select:
        return 'تحديد';
      case ModelingTool.column:
        return 'عمود';
      case ModelingTool.beam:
        return 'كمرة';
      case ModelingTool.slab:
        return 'بلاطة';
      case ModelingTool.wall:
        return 'جدار';
      case ModelingTool.foundation:
        return 'أساس';
    }
  }

  String _categoryLabel(String category) {
    switch (category) {
      case 'column':
        return 'أعمدة';
      case 'beam':
        return 'كمرات';
      case 'slab':
        return 'بلاطات';
      case 'wall':
        return 'جدران';
      case 'foundation':
        return 'أساسات';
      default:
        return category;
    }
  }

  void _handleElementSelected(String element) {
    setState(() {
      _selectedElement = element;
      _activeTool = ModelingTool.select;
    });
    if (MediaQuery.sizeOf(context).width < 940) {
      _showMobileProperties();
    }
  }

  void _toggleCategory(String category) {
    setState(() {
      if (!_hiddenCategories.add(category)) {
        _hiddenCategories.remove(category);
      }
    });
  }

  Future<void> _showMobileProperties() async {
    final selected = _selectedSnapshot;
    if (selected == null || !mounted) return;
    await showModalBottomSheet<void>(
      context: context,
      backgroundColor: const Color(0xff102635),
      showDragHandle: true,
      builder: (context) => SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 4, 20, 24),
          child: _buildElementProperties(selected, compact: true),
        ),
      ),
    );
  }

  void _saveProject() {
    setState(() => _projectDirty = false);
    _showMessage('تم حفظ بيانات النموذج الهندسي في المشروع.');
  }

  void _openFilePlaceholder() {
    _showMessage(
      'فتح الملفات سيعيد بناء النموذج من صيغة .civilx عند اكتمال موصل الملفات.',
    );
  }

  void _showUnitPicker() {
    showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xff102635),
        title: const Text(
          'وحدة العرض',
          style: TextStyle(color: Colors.white),
        ),
        content: DropdownButtonFormField<String>(
          initialValue: _displayUnit,
          dropdownColor: const Color(0xff183344),
          items: const [
            DropdownMenuItem(
              value: 'm',
              child: Text('متر (m)', style: TextStyle(color: Colors.white)),
            ),
            DropdownMenuItem(
              value: 'cm',
              child: Text('سنتيمتر (cm)', style: TextStyle(color: Colors.white)),
            ),
            DropdownMenuItem(
              value: 'mm',
              child: Text('مليمتر (mm)', style: TextStyle(color: Colors.white)),
            ),
          ],
          onChanged: (value) {
            if (value != null) {
              setState(() => _displayUnit = value);
              Navigator.of(context).pop();
            }
          },
        ),
      ),
    );
  }

  Future<void> _openNewProjectDialog() async {
    final draft = await showDialog<_ProjectDraft>(
      context: context,
      builder: (context) => _NewProjectDialog(
        initialName: _projectName,
        initialType: _projectType,
        initialLandArea: _landArea,
        projectTypes: _projectTypes,
      ),
    );
    if (draft == null || !mounted) return;

    if (!widget.kernelReady) {
      setState(() {
        _projectName = draft.name;
        _projectType = draft.type;
        _landArea = draft.landArea;
        _selectedElement = null;
        _snapshot = null;
      });
      _showMessage(
        'تم تجهيز بيانات المشروع في الواجهة. اتصل بالنواة لإنشاء النموذج الهندسي.',
        error: true,
      );
      return;
    }

    try {
      final snapshot = createWorkspaceSnapshot(
        name: draft.name,
        projectType: draft.type,
        landAreaM2: draft.landArea,
      );
      setState(() {
        _projectName = draft.name;
        _projectType = draft.type;
        _landArea = draft.landArea;
        _selectedElement = null;
        _snapshot = snapshot;
        _projectDirty = true;
      });
      _showMessage(
        'تم إنشاء النموذج الهندسي لـ "${snapshot.projectName}" · '
        '${snapshot.elements.length} عناصر إنشائية',
      );
    } catch (error) {
      _showMessage('تعذر إنشاء المشروع في النواة: $error', error: true);
    }
  }

  String _formatArea(double area) {
    return area == area.roundToDouble()
        ? area.toStringAsFixed(0)
        : area.toStringAsFixed(2);
  }

  @override
  Widget build(BuildContext context) {
    return Directionality(
      textDirection: TextDirection.rtl,
      child: Scaffold(
        backgroundColor: const Color(0xff07131f),
        body: SafeArea(
          child: Column(
            children: [
              _buildHeader(),
              Expanded(
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    final showInspector = constraints.maxWidth >= 940;
                    return Row(
                      children: [
                        if (showInspector) _buildInspector(),
                        Expanded(child: _buildViewport()),
                      ],
                    );
                  },
                ),
              ),
              _buildStatusBar(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return LayoutBuilder(
      builder: (context, constraints) {
        final compact = constraints.maxWidth < 720;
        final condensed = constraints.maxWidth < 1000;
        return Container(
          height: 72,
          padding: EdgeInsets.symmetric(horizontal: compact ? 10 : 22),
          decoration: const BoxDecoration(
            color: Color(0xff0b1d2c),
            border: Border(bottom: BorderSide(color: Color(0xff1d3547))),
          ),
          child: Row(
            children: [
              Container(
                width: 38,
                height: 38,
                decoration: BoxDecoration(
                  color: const Color(0xff18a6a6).withValues(alpha: .15),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(color: const Color(0xff27c6c0)),
                ),
                child: const Icon(Icons.architecture, color: Color(0xff54e0d7)),
              ),
              if (!compact) ...[
                const SizedBox(width: 12),
                const Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'منصة الهندسة المدنية',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 16,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    Text(
                      'مساحة العمل الهندسية',
                      style: TextStyle(color: Color(0xff8ea8b8), fontSize: 12),
                    ),
                  ],
                ),
              ],
              const Spacer(),
              if (!compact) _connectionBadge(),
              if (condensed)
                PopupMenuButton<String>(
                  tooltip: 'إجراءات المشروع',
                  color: const Color(0xff183344),
                  onSelected: (value) {
                    switch (value) {
                      case 'open':
                        _openFilePlaceholder();
                      case 'save':
                        _saveProject();
                      case 'units':
                        _showUnitPicker();
                    }
                  },
                  itemBuilder: (context) => const [
                    PopupMenuItem(
                      value: 'open',
                      child: Text('فتح مشروع'),
                    ),
                    PopupMenuItem(
                      value: 'save',
                      child: Text('حفظ المشروع'),
                    ),
                    PopupMenuItem(
                      value: 'units',
                      child: Text('وحدة العرض'),
                    ),
                  ],
                  icon: const Icon(
                    Icons.more_vert,
                    color: Color(0xffb7cad6),
                  ),
                ),
              if (!condensed) _buildHeaderAction(
                icon: Icons.folder_open_outlined,
                label: 'فتح',
                onPressed: _openFilePlaceholder,
              ),
              if (!condensed) _buildHeaderAction(
                icon: Icons.save_outlined,
                label: 'حفظ',
                onPressed: _saveProject,
              ),
              if (!condensed)
                _buildHeaderAction(
                  icon: Icons.undo_outlined,
                  label: 'تراجع',
                  onPressed: () => _showMessage(
                    'لا توجد تعديلات هندسية للتراجع عنها في هذه الجلسة.',
                  ),
                ),
              if (!condensed)
                _buildHeaderAction(
                  icon: Icons.redo_outlined,
                  label: 'إعادة',
                  onPressed: () => _showMessage(
                    'لا توجد تعديلات هندسية لإعادتها في هذه الجلسة.',
                  ),
                ),
              IconButton(
                tooltip: 'إنشاء مشروع جديد',
                onPressed: _openNewProjectDialog,
                icon: const Icon(
                  Icons.create_new_folder_outlined,
                  color: Color(0xffb7cad6),
                ),
              ),
              if (!condensed) ...[
                const SizedBox(width: 4),
                IconButton(
                  tooltip: 'فحص اتصال النواة',
                  onPressed: _checkKernel,
                  icon: const Icon(Icons.sync, color: Color(0xffb7cad6)),
                ),
                const SizedBox(width: 4),
                IconButton(
                  tooltip: 'الإعدادات',
                  onPressed: _showUnitPicker,
                  icon: const Icon(
                    Icons.settings_outlined,
                    color: Color(0xffb7cad6),
                  ),
                ),
              ],
              const SizedBox(width: 8),
              CircleAvatar(
                radius: 18,
                backgroundColor: const Color(0xff24536b),
                child: const Text(
                  'م',
                  style: TextStyle(
                    color: Colors.white,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildHeaderAction({
    required IconData icon,
    required String label,
    required VoidCallback onPressed,
  }) {
    return Tooltip(
      message: label,
      child: IconButton(
        onPressed: onPressed,
        icon: Icon(icon, color: const Color(0xffb7cad6)),
      ),
    );
  }

  Widget _connectionBadge() {
    final connected = widget.kernelReady;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 7),
      decoration: BoxDecoration(
        color: connected ? const Color(0xff123d3b) : const Color(0xff402b25),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(
          color: connected ? const Color(0xff287d73) : const Color(0xff85533a),
        ),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 7,
            height: 7,
            decoration: BoxDecoration(
              color: connected
                  ? const Color(0xff53d8aa)
                  : const Color(0xffffad70),
              shape: BoxShape.circle,
            ),
          ),
          const SizedBox(width: 7),
          Text(
            connected ? 'النواة متصلة' : 'النواة غير متصلة',
            style: TextStyle(
              color: connected
                  ? const Color(0xff8de6cc)
                  : const Color(0xffffc494),
              fontSize: 12,
              fontWeight: FontWeight.w600,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInspector() {
    final snapshot = _snapshot;
    final levels = snapshot?.levels ?? const <LevelSnapshot>[];
    final grids = snapshot?.grids ?? const <GridSnapshot>[];
    final elements = snapshot?.elements ?? const <ElementSnapshot>[];
    final columns = elements
        .where((element) => element.category == 'column')
        .length;
    final beams = elements
        .where((element) => element.category == 'beam')
        .length;
    final slabs = elements
        .where((element) => element.category == 'slab')
        .length;
    return Container(
      width: 276,
      decoration: const BoxDecoration(
        color: Color(0xff0b1d2c),
        border: Border(left: BorderSide(color: Color(0xff1d3547))),
      ),
      child: ListView(
        padding: const EdgeInsets.fromLTRB(18, 22, 18, 18),
        children: [
          const Text(
            'مستكشف المشروع',
            style: TextStyle(
              color: Colors.white,
              fontSize: 15,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 5),
          Text(
            '$_projectType · مساحة ${_formatArea(_landArea)} م²',
            style: TextStyle(color: Color(0xff7793a3), fontSize: 11),
          ),
          const SizedBox(height: 4),
          Text(
            '${_projectDirty ? '● ' : ''}$_projectName',
            style: const TextStyle(
              color: Color(0xffd5e1e7),
              fontSize: 13,
              fontWeight: FontWeight.w600,
            ),
            overflow: TextOverflow.ellipsis,
          ),
          const SizedBox(height: 12),
          _buildToolPalette(compact: false),
          const SizedBox(height: 22),
          _treeSection(
            icon: Icons.layers_outlined,
            title: 'المستويات',
            value: '${levels.length}',
            children: [
              for (final level in levels)
                '${level.name}  ·  ${level.elevationM.toStringAsFixed(2)} m',
            ],
          ),
          _treeSection(
            icon: Icons.grid_4x4,
            title: 'الشبكات',
            value: '${grids.length}',
            children: [for (final grid in grids) grid.name],
          ),
          _treeSection(
            icon: Icons.account_tree_outlined,
            title: 'العناصر الإنشائية',
            value: '${elements.length}',
            children: [
              'الأعمدة  ·  $columns',
              'الكمرات  ·  $beams',
              'البلاطات  ·  $slabs',
            ],
          ),
          const SizedBox(height: 18),
          _buildVisibilitySection(),
          const SizedBox(height: 18),
          Container(
            padding: const EdgeInsets.all(13),
            decoration: BoxDecoration(
              color: const Color(0xff102a3a),
              borderRadius: BorderRadius.circular(10),
              border: Border.all(color: const Color(0xff23465b)),
            ),
            child: const Row(
              children: [
                Icon(Icons.info_outline, color: Color(0xff54e0d7), size: 18),
                SizedBox(width: 9),
                Expanded(
                  child: Text(
                    'اختر أي عنصر من المشهد لعرض خصائصه.',
                    style: TextStyle(
                      color: Color(0xffa7bdc9),
                      fontSize: 11,
                      height: 1.5,
                    ),
                  ),
                ),
              ],
            ),
          ),
          if (_selectedElement != null) ...[
            const SizedBox(height: 24),
             if (_selectedSnapshot case final selected?)
               _buildElementProperties(selected),
          ],
        ],
      ),
    );
  }

  Widget _buildToolPalette({required bool compact}) {
    final tools = [
      (ModelingTool.select, Icons.near_me_outlined),
      (ModelingTool.column, Icons.view_column_outlined),
      (ModelingTool.beam, Icons.horizontal_rule),
      (ModelingTool.slab, Icons.layers_outlined),
      (ModelingTool.wall, Icons.view_agenda_outlined),
      (ModelingTool.foundation, Icons.foundation_outlined),
    ];
    return Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: const Color(0xff102a3a),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xff23465b)),
      ),
      child: compact
          ? Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                for (final tool in tools) _toolPaletteButton(tool.$1, tool.$2),
              ],
            )
          : Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                const Text(
                  'أدوات النمذجة',
                  style: TextStyle(
                    color: Color(0xffd5e1e7),
                    fontSize: 12,
                    fontWeight: FontWeight.w700,
                  ),
                ),
                const SizedBox(height: 6),
                for (final tool in tools)
                  _toolPaletteButton(tool.$1, tool.$2, expanded: true),
              ],
            ),
    );
  }

  Widget _toolPaletteButton(
    ModelingTool tool,
    IconData icon, {
    bool expanded = false,
  }) {
    final active = _activeTool == tool;
    final button = InkWell(
      onTap: () => _selectTool(tool),
      borderRadius: BorderRadius.circular(7),
      child: Container(
        padding: EdgeInsets.symmetric(
          horizontal: expanded ? 9 : 7,
          vertical: expanded ? 8 : 7,
        ),
        margin: EdgeInsets.only(bottom: expanded ? 3 : 0),
        decoration: BoxDecoration(
          color: active ? const Color(0xff164b56) : Colors.transparent,
          borderRadius: BorderRadius.circular(7),
        ),
        child: Row(
          mainAxisAlignment: expanded
              ? MainAxisAlignment.start
              : MainAxisAlignment.center,
          children: [
            Icon(
              icon,
              size: 17,
              color: active
                  ? const Color(0xff62eee2)
                  : const Color(0xffa5bbc6),
            ),
            if (expanded) ...[
              const SizedBox(width: 9),
              Text(
                _toolLabel(tool),
                style: TextStyle(
                  color: active
                      ? const Color(0xff62eee2)
                      : const Color(0xffa5bbc6),
                  fontSize: 11,
                  fontWeight: active ? FontWeight.w700 : FontWeight.w500,
                ),
              ),
            ],
          ],
        ),
      ),
    );
    return expanded
        ? Tooltip(message: _toolLabel(tool), child: button)
        : Padding(
            padding: const EdgeInsets.symmetric(horizontal: 1),
            child: Tooltip(message: _toolLabel(tool), child: button),
          );
  }

  Widget _buildVisibilitySection() {
    const categories = ['column', 'beam', 'slab', 'wall', 'foundation'];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Text(
          'مرشحات العرض',
          style: TextStyle(
            color: Color(0xffd5e1e7),
            fontSize: 12,
            fontWeight: FontWeight.w700,
          ),
        ),
        const SizedBox(height: 5),
        for (final category in categories)
          InkWell(
            onTap: () => _toggleCategory(category),
            borderRadius: BorderRadius.circular(6),
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 3),
              child: Row(
                children: [
                  Icon(
                    _hiddenCategories.contains(category)
                        ? Icons.visibility_off_outlined
                        : Icons.visibility_outlined,
                    size: 16,
                    color: _hiddenCategories.contains(category)
                        ? const Color(0xff668391)
                        : const Color(0xff54e0d7),
                  ),
                  const SizedBox(width: 8),
                  Text(
                    _categoryLabel(category),
                    style: TextStyle(
                      color: _hiddenCategories.contains(category)
                          ? const Color(0xff668391)
                          : const Color(0xffa7bdc9),
                      fontSize: 11,
                    ),
                  ),
                ],
              ),
            ),
          ),
        InkWell(
          onTap: () => setState(() => _showGrid = !_showGrid),
          borderRadius: BorderRadius.circular(6),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 3),
            child: Row(
              children: [
                Icon(
                  _showGrid
                      ? Icons.grid_4x4
                      : Icons.grid_off,
                  size: 16,
                  color: _showGrid
                      ? const Color(0xff54e0d7)
                      : const Color(0xff668391),
                ),
                const SizedBox(width: 8),
                Text(
                  'الشبكة الهندسية',
                  style: TextStyle(
                    color: _showGrid
                        ? const Color(0xffa7bdc9)
                        : const Color(0xff668391),
                    fontSize: 11,
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildElementProperties(
    ElementSnapshot element, {
    bool compact = false,
  }) {
    final unit = _displayUnit;
    String length(double value) {
      final converted = switch (unit) {
        'mm' => value * 1000,
        'cm' => value * 100,
        _ => value,
      };
      return '${converted.toStringAsFixed(2)} $unit';
    }

    final isColumn = element.category == 'column';
    final isBeam = element.category == 'beam';
    final isSlab = element.category == 'slab';
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Row(
          children: [
            const Icon(Icons.tune, color: Color(0xff54e0d7), size: 17),
            const SizedBox(width: 8),
            Text(
              'خصائص ${element.name}',
              style: const TextStyle(
                color: Color(0xffd5e1e7),
                fontSize: 12,
                fontWeight: FontWeight.w700,
              ),
            ),
          ],
        ),
        const SizedBox(height: 10),
        _propertyRow('النوع', _categoryLabel(element.category)),
        if (isColumn) ...[
          _propertyRow('الموقع', '${length(element.x)} · ${length(element.y)}'),
          _propertyRow('الارتفاع', length(element.topZ - element.z)),
          _propertyRow(
            'القطاع',
            '${length(element.width)} × ${length(element.depth)}',
          ),
          _propertyRow('المستوى', 'Level 1 → Level 2'),
          _propertyRow('المادة', 'خرسانة C30'),
        ],
        if (isBeam) ...[
          _propertyRow('البداية', _pointLabel(element.start, length)),
          _propertyRow('النهاية', _pointLabel(element.end, length)),
          _propertyRow(
            'القطاع',
            '${length(element.width)} × ${length(element.depth)}',
          ),
          _propertyRow('المستوى', 'Level 2'),
        ],
        if (isSlab) ...[
          _propertyRow('المستوى', 'Level 2'),
          _propertyRow('السمك', length(element.thickness)),
          _propertyRow('الرؤوس', '${element.boundary.length} نقاط هندسية'),
        ],
        if (!compact)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: OutlinedButton.icon(
              onPressed: () => _showMessage(
                'تعديل الخصائص سيُرسل كأمر هندسي إلى النواة في الخطوة التالية.',
              ),
              icon: const Icon(Icons.edit_outlined, size: 15),
              label: const Text('تعديل الخصائص'),
            ),
          ),
      ],
    );
  }

  String _pointLabel(PointSnapshot point, String Function(double) length) {
    return '${length(point.x)}, ${length(point.y)}, ${length(point.z)}';
  }

  Widget _propertyRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 64,
            child: Text(
              label,
              style: const TextStyle(color: Color(0xff668391), fontSize: 10),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(color: Color(0xffa7bdc9), fontSize: 10),
            ),
          ),
        ],
      ),
    );
  }

  Widget _treeSection({
    required IconData icon,
    required String title,
    required String value,
    required List<String> children,
  }) {
    return ExpansionTile(
      tilePadding: EdgeInsets.zero,
      childrenPadding: const EdgeInsets.only(right: 31, bottom: 8),
      initiallyExpanded: title == 'العناصر الإنشائية',
      iconColor: const Color(0xff7793a3),
      collapsedIconColor: const Color(0xff7793a3),
      title: Row(
        children: [
          Icon(icon, size: 18, color: const Color(0xff54e0d7)),
          const SizedBox(width: 10),
          Expanded(
            child: Text(
              title,
              style: const TextStyle(color: Color(0xffd5e1e7), fontSize: 13),
            ),
          ),
          Text(
            value,
            style: const TextStyle(color: Color(0xff6d8999), fontSize: 11),
          ),
        ],
      ),
      children: [
        for (final child in children)
          Align(
            alignment: Alignment.centerRight,
            child: Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: Text(
                child,
                style: const TextStyle(color: Color(0xff91aab8), fontSize: 11),
              ),
            ),
          ),
      ],
    );
  }

  Widget _buildViewport() {
    return Container(
      margin: const EdgeInsets.all(14),
      clipBehavior: Clip.antiAlias,
      decoration: BoxDecoration(
        color: const Color(0xff091722),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xff1d3547)),
      ),
      child: Stack(
        children: [
          EngineeringViewport(
            interaction: _interaction,
            projection: _projection,
            snapshot: _snapshot,
            hiddenCategories: _hiddenCategories,
            showGrid: _showGrid,
            resetToken: _viewResetToken,
            onElementSelected: _handleElementSelected,
          ),
          Positioned(top: 16, right: 16, child: _viewportToolbar()),
          Positioned(top: 16, left: 16, child: _viewCube()),
          Positioned(bottom: 16, right: 16, child: _viewportLegend()),
          Positioned(
            bottom: 16,
            left: 16,
            child: _buildToolPalette(compact: true),
          ),
        ],
      ),
    );
  }

  Widget _viewportToolbar() {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: const Color(0xdd0b1d2c),
        borderRadius: BorderRadius.circular(9),
        border: Border.all(color: const Color(0xff2a4658)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _toolButton(
            icon: Icons.threesixty,
            label: 'دوران',
            active: _interaction == ViewportInteraction.orbit,
            onPressed: () =>
                setState(() => _interaction = ViewportInteraction.orbit),
          ),
          _toolButton(
            icon: Icons.pan_tool_outlined,
            label: 'تحريك',
            active: _interaction == ViewportInteraction.pan,
            onPressed: () =>
                setState(() => _interaction = ViewportInteraction.pan),
          ),
          Container(width: 1, height: 24, color: const Color(0xff2a4658)),
          _toolButton(
            icon: _projection == ViewportProjection.perspective
                ? Icons.camera_alt_outlined
                : Icons.straighten,
            label: _projection == ViewportProjection.perspective
                ? 'منظور'
                : 'متعامد',
            active: false,
            onPressed: () => setState(
              () => _projection = _projection == ViewportProjection.perspective
                  ? ViewportProjection.orthographic
                  : ViewportProjection.perspective,
            ),
          ),
          _toolButton(
            icon: Icons.center_focus_strong,
            label: 'ملاءمة',
            active: false,
            onPressed: () => setState(() => _viewResetToken++),
          ),
        ],
      ),
    );
  }

  Widget _toolButton({
    required IconData icon,
    required String label,
    required bool active,
    required VoidCallback onPressed,
  }) {
    return Tooltip(
      message: label,
      child: InkWell(
        onTap: onPressed,
        borderRadius: BorderRadius.circular(8),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 9),
          decoration: BoxDecoration(
            color: active ? const Color(0xff164b56) : Colors.transparent,
            borderRadius: BorderRadius.circular(8),
          ),
          child: Icon(
            icon,
            size: 19,
            color: active ? const Color(0xff62eee2) : const Color(0xffa5bbc6),
          ),
        ),
      ),
    );
  }

  Widget _viewCube() {
    return Container(
      width: 74,
      height: 74,
      decoration: BoxDecoration(
        color: const Color(0xc90b1d2c),
        borderRadius: BorderRadius.circular(10),
        border: Border.all(color: const Color(0xff2a4658)),
      ),
      child: Stack(
        alignment: Alignment.center,
        children: [
          Transform(
            alignment: Alignment.center,
            transform: Matrix4.identity()
              ..setEntry(3, 2, 0.002)
              ..rotateX(-.42)
              ..rotateY(.52),
            child: Container(
              width: 34,
              height: 34,
              decoration: BoxDecoration(
                color: const Color(0xff1b6b78),
                border: Border.all(color: const Color(0xff7fe1d8)),
              ),
              child: const Center(
                child: Text(
                  'TOP',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 8,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ),
            ),
          ),
          const Positioned(
            right: 5,
            top: 4,
            child: Text(
              'Z',
              style: TextStyle(
                color: Color(0xff65a7ff),
                fontSize: 10,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
          const Positioned(
            left: 5,
            bottom: 6,
            child: Text(
              'X',
              style: TextStyle(
                color: Color(0xffff8b80),
                fontSize: 10,
                fontWeight: FontWeight.w700,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _viewportLegend() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 8),
      decoration: BoxDecoration(
        color: const Color(0xc90b1d2c),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: const Color(0xff2a4658)),
      ),
      child: const Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          _LegendDot(color: Color(0xfff06b62), label: 'X'),
          SizedBox(width: 10),
          _LegendDot(color: Color(0xff62d49b), label: 'Y'),
          SizedBox(width: 10),
          _LegendDot(color: Color(0xff65a7ff), label: 'Z'),
        ],
      ),
    );
  }

  Widget _buildStatusBar() {
    return Container(
      height: 32,
      padding: const EdgeInsets.symmetric(horizontal: 18),
      color: const Color(0xff06101a),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final compact = constraints.maxWidth < 620;
          return Row(
            children: [
              const Icon(
                Icons.mouse_outlined,
                color: Color(0xff668391),
                size: 14,
              ),
              const SizedBox(width: 7),
              Expanded(
                child: Text(
                  compact ? 'Orbit  ·  Zoom  ·  Pan' : 'اسحب للدوران  ·  عجلة الماوس للتكبير  ·  اختر التحريك لتحريك المشهد',
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: Color(0xff668391),
                    fontSize: 10,
                  ),
                ),
              ),
              if (!compact)
                Text(
                  'الوحدات: $_displayUnit  ·  شبكة: 1.00',
                  style: TextStyle(color: Color(0xff668391), fontSize: 10),
                ),
              if (!compact) ...[
                const SizedBox(width: 12),
                Text(
                  _projectDirty ? 'غير محفوظ' : 'محفوظ',
                  style: TextStyle(
                    color: _projectDirty
                        ? const Color(0xffffc494)
                        : const Color(0xff72d6b5),
                    fontSize: 10,
                  ),
                ),
              ],
            ],
          );
        },
      ),
    );
  }
}

class _LegendDot extends StatelessWidget {
  const _LegendDot({required this.color, required this.label});

  final Color color;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 7,
          height: 7,
          decoration: BoxDecoration(color: color, shape: BoxShape.circle),
        ),
        const SizedBox(width: 4),
        Text(
          label,
          style: TextStyle(
            color: color,
            fontSize: 10,
            fontWeight: FontWeight.w700,
          ),
        ),
      ],
    );
  }
}

class _ProjectDraft {
  const _ProjectDraft({
    required this.name,
    required this.type,
    required this.landArea,
  });

  final String name;
  final String type;
  final double landArea;
}

class _NewProjectDialog extends StatefulWidget {
  const _NewProjectDialog({
    required this.initialName,
    required this.initialType,
    required this.initialLandArea,
    required this.projectTypes,
  });

  final String initialName;
  final String initialType;
  final double initialLandArea;
  final List<String> projectTypes;

  @override
  State<_NewProjectDialog> createState() => _NewProjectDialogState();
}

class _NewProjectDialogState extends State<_NewProjectDialog> {
  late final TextEditingController _nameController;
  late final TextEditingController _areaController;
  late String _selectedType;
  String? _errorText;

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController(text: widget.initialName);
    _areaController = TextEditingController(
      text: widget.initialLandArea.toStringAsFixed(0),
    );
    _selectedType = widget.initialType;
  }

  @override
  void dispose() {
    _nameController.dispose();
    _areaController.dispose();
    super.dispose();
  }

  void _submit() {
    final name = _nameController.text.trim();
    final area = double.tryParse(_areaController.text.trim());
    if (name.isEmpty) {
      setState(() => _errorText = 'اكتب اسمًا للمشروع أولًا.');
      return;
    }
    if (area == null || area <= 0) {
      setState(() => _errorText = 'أدخل مساحة صحيحة أكبر من صفر.');
      return;
    }
    Navigator.of(context)
        .pop(_ProjectDraft(name: name, type: _selectedType, landArea: area));
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final compact = MediaQuery.sizeOf(context).width < 520;
    return AlertDialog(
      backgroundColor: const Color(0xff102635),
      surfaceTintColor: Colors.transparent,
      title: const Row(
        children: [
          Icon(Icons.create_new_folder_outlined, color: Color(0xff54e0d7)),
          SizedBox(width: 10),
          Text(
            'إنشاء مشروع جديد',
            style: TextStyle(color: Colors.white, fontSize: 18),
          ),
        ],
      ),
      content: SizedBox(
        width: compact ? null : 480,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const Text(
                'أدخل بيانات المشروع الأساسية للبدء في مساحة عمل جديدة.',
                style: TextStyle(color: Color(0xffa7bdc9), fontSize: 12),
              ),
              const SizedBox(height: 22),
              TextField(
                controller: _nameController,
                autofocus: true,
                textInputAction: TextInputAction.next,
                decoration: _inputDecoration(
                  label: 'اسم المشروع',
                  hint: 'مثال: جسر وادي حضرموت',
                  icon: Icons.title,
                ),
              ),
              const SizedBox(height: 14),
              DropdownButtonFormField<String>(
                initialValue: _selectedType,
                dropdownColor: const Color(0xff183344),
                decoration: _inputDecoration(
                  label: 'نوع المشروع',
                  hint: '',
                  icon: Icons.category_outlined,
                ),
                items: [
                  for (final type in widget.projectTypes)
                    DropdownMenuItem(
                      value: type,
                      child: Text(
                        type,
                        style: const TextStyle(color: Colors.white),
                      ),
                    ),
                ],
                onChanged: (value) {
                  if (value != null) setState(() => _selectedType = value);
                },
              ),
              const SizedBox(height: 14),
              TextField(
                controller: _areaController,
                keyboardType: const TextInputType.numberWithOptions(
                  decimal: true,
                ),
                textInputAction: TextInputAction.done,
                onSubmitted: (_) => _submit(),
                decoration: _inputDecoration(
                  label: 'المساحة التقديرية لقطعة الأرض',
                  hint: '1200',
                  icon: Icons.square_foot,
                ).copyWith(suffixText: 'م²'),
              ),
              if (_errorText != null) ...[
                const SizedBox(height: 12),
                Text(
                  _errorText!,
                  style: TextStyle(
                    color: theme.colorScheme.error,
                    fontSize: 12,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
      actionsPadding: const EdgeInsets.fromLTRB(20, 0, 20, 18),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('إلغاء'),
        ),
        FilledButton.icon(
          onPressed: _submit,
          icon: const Icon(Icons.add, size: 18),
          label: const Text('إنشاء المشروع'),
        ),
      ],
    );
  }

  InputDecoration _inputDecoration({
    required String label,
    required String hint,
    required IconData icon,
  }) {
    return InputDecoration(
      labelText: label,
      hintText: hint,
      prefixIcon: Icon(icon, color: const Color(0xff54e0d7)),
      labelStyle: const TextStyle(color: Color(0xffa7bdc9)),
      hintStyle: const TextStyle(color: Color(0xff6f8c9d)),
      filled: true,
      fillColor: const Color(0xff0b1d2c),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: Color(0xff2a4658)),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: Color(0xff2a4658)),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: Color(0xff54e0d7), width: 1.5),
      ),
    );
  }
}
