import 'package:flutter/material.dart';

/// Adaptive floating toolbar that works on mobile (bottom), tablet (side), desktop (ribbon).
/// Provides quick access to engineering tools, snap, and view controls.
class FloatingToolbar extends StatefulWidget {
  const FloatingToolbar({
    required this.activeTool,
    required this.onToolSelected,
    required this.snapEnabled,
    required this.onSnapToggle,
    required this.onViewChanged,
    this.position = ToolbarPosition.bottom,
    super.key,
  });

  final String activeTool;
  final ValueChanged<String> onToolSelected;
  final bool snapEnabled;
  final VoidCallback onSnapToggle;
  final ValueChanged<String> onViewChanged;
  final ToolbarPosition position;

  @override
  State<FloatingToolbar> createState() => _FloatingToolbarState();
}

enum ToolbarPosition { bottom, side, top }

class _FloatingToolbarState extends State<FloatingToolbar>
    with SingleTickerProviderStateMixin {
  bool _expanded = false;
  late AnimationController _expandController;
  late Animation<double> _expandAnimation;

  @override
  void initState() {
    super.initState();
    _expandController = AnimationController(
      duration: const Duration(milliseconds: 200),
      vsync: this,
    );
    _expandAnimation = CurvedAnimation(
      parent: _expandController,
      curve: Curves.easeOutCubic,
    );
  }

  @override
  void dispose() {
    _expandController.dispose();
    super.dispose();
  }

  void _toggleExpand() {
    setState(() => _expanded = !_expanded);
    if (_expanded) {
      _expandController.forward();
    } else {
      _expandController.reverse();
    }
  }

  @override
  Widget build(BuildContext context) {
    final isSide = widget.position == ToolbarPosition.side;
    final tools = _getTools();

    return AnimatedBuilder(
      animation: _expandAnimation,
      builder: (context, child) {
        return Container(
          decoration: BoxDecoration(
            color: const Color(0xdd0b1d2c),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: const Color(0xff2a5260)),
            boxShadow: const [
              BoxShadow(
                color: Color(0x40000000),
                blurRadius: 12,
                offset: Offset(0, 4),
              ),
            ],
          ),
          padding: const EdgeInsets.all(6),
          child: isSide
              ? _buildSideLayout(tools)
              : _buildBottomLayout(tools),
        );
      },
    );
  }

  Widget _buildBottomLayout(List<ToolItem> tools) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Expanded tools section
        if (_expanded) ...[
          SizedBox(
            height: 180 * _expandAnimation.value,
            child: Opacity(
              opacity: _expandAnimation.value,
              child: _buildExpandedTools(tools),
            ),
          ),
          const SizedBox(height: 4),
        ],
        // Main toolbar row
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Toggle expand
            _ToolbarButton(
              icon: _expanded ? Icons.keyboard_arrow_down : Icons.keyboard_arrow_up,
              label: '',
              isActive: false,
              onTap: _toggleExpand,
            ),
            const SizedBox(width: 2),
            // Primary tools
            for (final tool in tools.take(_expanded ? tools.length : 5))
              _ToolbarButton(
                icon: tool.icon,
                label: tool.label,
                isActive: widget.activeTool == tool.id,
                onTap: () => widget.onToolSelected(tool.id),
              ),
            if (!_expanded && tools.length > 5)
              _ToolbarButton(
                icon: Icons.more_horiz,
                label: 'المزيد',
                isActive: false,
                onTap: _toggleExpand,
              ),
            const SizedBox(width: 4),
            // Snap toggle
            Container(
              width: 1,
              height: 24,
              color: const Color(0xff2a5260),
            ),
            const SizedBox(width: 4),
            _ToolbarButton(
              icon: widget.snapEnabled ? Icons.grid_on : Icons.grid_off,
              label: 'التقاط',
              isActive: widget.snapEnabled,
              onTap: widget.onSnapToggle,
            ),
            // View controls
            _ToolbarButton(
              icon: Icons.view_in_ar,
              label: 'عرض',
              isActive: false,
              onTap: () => _showViewPicker(),
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildSideLayout(List<ToolItem> tools) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        for (final tool in tools)
          _ToolbarButton(
            icon: tool.icon,
            label: tool.label,
            isActive: widget.activeTool == tool.id,
            onTap: () => widget.onToolSelected(tool.id),
          ),
        const SizedBox(height: 4),
        Container(
          width: 24,
          height: 1,
          color: const Color(0xff2a5260),
        ),
        const SizedBox(height: 4),
        _ToolbarButton(
          icon: widget.snapEnabled ? Icons.grid_on : Icons.grid_off,
          label: 'التقاط',
          isActive: widget.snapEnabled,
          onTap: widget.onSnapToggle,
        ),
      ],
    );
  }

  Widget _buildExpandedTools(List<ToolItem> tools) {
    final sections = [
      _ToolSection(
        label: 'إنشاء',
        tools: tools.where((t) => t.section == 'create').toList(),
      ),
      _ToolSection(
        label: 'تعديل',
        tools: tools.where((t) => t.section == 'edit').toList(),
      ),
      _ToolSection(
        label: 'عرض',
        tools: tools.where((t) => t.section == 'view').toList(),
      ),
    ];

    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          for (final section in sections) ...[
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                  child: Text(
                    section.label,
                    style: const TextStyle(
                      color: Color(0xff7793a3),
                      fontSize: 9,
                      fontWeight: FontWeight.w700,
                    ),
                  ),
                ),
                Wrap(
                  spacing: 4,
                  runSpacing: 4,
                  children: [
                    for (final tool in section.tools)
                      _ToolbarButton(
                        icon: tool.icon,
                        label: tool.label,
                        isActive: widget.activeTool == tool.id,
                        onTap: () => widget.onToolSelected(tool.id),
                      ),
                  ],
                ),
              ],
            ),
            const SizedBox(width: 16),
          ],
        ],
      ),
    );
  }

  void _showViewPicker() {
    showModalBottomSheet(
      context: context,
      backgroundColor: const Color(0xff102635),
      builder: (context) => Container(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text(
              'عرض',
              style: TextStyle(
                color: Colors.white,
                fontSize: 16,
                fontWeight: FontWeight.w700,
              ),
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                for (final view in [
                  ('3d', 'ثلاثي الأبعاد', Icons.view_in_ar),
                  ('top', 'من أعلى', Icons.flip),
                  ('front', 'الواجهة', Icons.view_carousel),
                  ('left', 'اليسار', Icons.view_sidebar),
                  ('iso', 'متساوية البعد', Icons.view_compact),
                ])
                  ActionChip(
                    label: Text(view.$2, style: const TextStyle(color: Colors.white, fontSize: 12)),
                    avatar: Icon(view.$3, color: const Color(0xff54e0d7), size: 16),
                    backgroundColor: const Color(0xff183344),
                    side: const BorderSide(color: Color(0xff2a5260)),
                    onPressed: () {
                      widget.onViewChanged(view.$1);
                      Navigator.pop(context);
                    },
                  ),
              ],
            ),
            const SizedBox(height: 12),
            // Projection toggle
            Row(
              children: [
                const Text('العرض: ', style: TextStyle(color: Color(0xffa7bdc9), fontSize: 12)),
                const SizedBox(width: 8),
                _ProjectionToggle(
                  onSelected: (mode) {
                    widget.onViewChanged(mode);
                    Navigator.pop(context);
                  },
                ),
              ],
            ),
            const SizedBox(height: 16),
          ],
        ),
      ),
    );
  }

  List<ToolItem> _getTools() {
    return [
      // Create tools
      ToolItem('select', Icons.near_me_outlined, 'تحديد', 'create'),
      ToolItem('column', Icons.view_column_outlined, 'عمود', 'create'),
      ToolItem('beam', Icons.horizontal_rule, 'كمرة', 'create'),
      ToolItem('slab', Icons.layers_outlined, 'بلاطة', 'create'),
      ToolItem('wall', Icons.view_agenda_outlined, 'جدار', 'create'),
      ToolItem('foundation', Icons.foundation_outlined, 'أساس', 'create'),
      ToolItem('roof', Icons.home_outlined, 'سقف', 'create'),
      // Edit tools
      ToolItem('move', Icons.open_with, 'نقل', 'edit'),
      ToolItem('copy', Icons.copy, 'نسخ', 'edit'),
      ToolItem('rotate', Icons.rotate_right, 'تدوير', 'edit'),
      ToolItem('mirror', Icons.flip, 'انعكاس', 'edit'),
      ToolItem('delete', Icons.delete_outline, 'حذف', 'edit'),
      // View tools
      ToolItem('measure', Icons.straighten, 'قياس', 'view'),
      ToolItem('section', Icons.crop, 'مقطع', 'view'),
    ];
  }
}

class ToolItem {
  const ToolItem(this.id, this.icon, this.label, this.section);
  final String id;
  final IconData icon;
  final String label;
  final String section;
}

class _ToolbarButton extends StatelessWidget {
  const _ToolbarButton({
    required this.icon,
    required this.label,
    required this.isActive,
    required this.onTap,
  });

  final IconData icon;
  final String label;
  final bool isActive;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: label,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(6),
        child: Container(
          width: 36,
          height: 36,
          margin: const EdgeInsets.all(2),
          decoration: BoxDecoration(
            color: isActive ? const Color(0xff164b56) : Colors.transparent,
            borderRadius: BorderRadius.circular(6),
            border: isActive
                ? Border.all(color: const Color(0xff54e0d7).withOpacity(0.3))
                : null,
          ),
          child: Icon(
            icon,
            size: 18,
            color: isActive ? const Color(0xff62eee2) : const Color(0xffa5bbc6),
          ),
        ),
      ),
    );
  }
}

class _ToolSection {
  const _ToolSection({required this.label, required this.tools});
  final String label;
  final List<ToolItem> tools;
}

class _ProjectionToggle extends StatelessWidget {
  const _ProjectionToggle({required this.onSelected});
  final ValueChanged<String> onSelected;

  @override
  Widget build(BuildContext context) {
    return SegmentedButton<String>(
      segments: const [
        ButtonSegment(
          value: 'perspective',
          label: Text('منظور', style: TextStyle(fontSize: 11)),
          icon: Icon(Icons.pets, size: 14),
        ),
        ButtonSegment(
          value: 'orthographic',
          label: Text('متعامد', style: TextStyle(fontSize: 11)),
          icon: Icon(Icons.square_foot, size: 14),
        ),
      ],
      onSelectionChanged: (values) => onSelected(values.first),
      style: ButtonStyle(
        backgroundColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return const Color(0xff164b56);
          }
          return const Color(0xff0b1d2c);
        }),
        foregroundColor: WidgetStateProperty.resolveWith((states) {
          if (states.contains(WidgetState.selected)) {
            return const Color(0xff62eee2);
          }
          return const Color(0xffa5bbc6);
        }),
        side: WidgetStateProperty.all(
          const BorderSide(color: Color(0xff2a5260)),
        ),
      ),
    );
  }
}
