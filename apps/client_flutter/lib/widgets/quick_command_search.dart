import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Quick command search overlay - press Ctrl+K or Cmd+K to open.
/// Allows typing tool names to quickly access them.
class QuickCommandSearch extends StatefulWidget {
  const QuickCommandSearch({
    required this.onCommandSelected,
    required this.visible,
    required this.onDismiss,
    super.key,
  });

  final ValueChanged<String> onCommandSelected;
  final bool visible;
  final VoidCallback onDismiss;

  @override
  State<QuickCommandSearch> createState() => _QuickCommandSearchState();
}

class _QuickCommandSearchState extends State<QuickCommandSearch>
    with SingleTickerProviderStateMixin {
  final TextEditingController _searchController = TextEditingController();
  final FocusNode _focusNode = FocusNode();
  late AnimationController _controller;
  late Animation<double> _scale;
  List<CommandItem> _filtered = [];
  int _selectedIndex = 0;

  static const _commands = [
    CommandItem('column', 'إنشاء عمود', 'column', Icons.view_column_outlined),
    CommandItem('beam', 'إنشاء كمرة', 'beam', Icons.horizontal_rule),
    CommandItem('slab', 'إنشاء بلاطة', 'slab', Icons.layers_outlined),
    CommandItem('wall', 'إنشاء جدار', 'wall', Icons.view_agenda_outlined),
    CommandItem('foundation', 'إنشاء أساس', 'foundation', Icons.foundation_outlined),
    CommandItem('roof', 'إنشاء سقف', 'roof', Icons.home_outlined),
    CommandItem('move', 'نقل عنصر', 'move', Icons.open_with),
    CommandItem('copy', 'نسخ عنصر', 'copy', Icons.copy),
    CommandItem('rotate', 'تدوير', 'rotate', Icons.rotate_right),
    CommandItem('mirror', 'انعكاس', 'mirror', Icons.flip),
    CommandItem('delete', 'حذف عنصر', 'delete', Icons.delete_outline),
    CommandItem('select', 'تحديد', 'select', Icons.near_me_outlined),
    CommandItem('measure', 'أداة القياس', 'measure', Icons.straighten),
    CommandItem('grid', 'إظهار/إخفاء الشبكة', 'grid', Icons.grid_on),
    CommandItem('levels', 'إنشاء مستويات', 'levels', Icons.layers_outlined),
    CommandItem('copy_levels', 'نسخ إلى مستويات', 'copy_levels', Icons.copy_all),
    CommandItem('undo', 'تراجع', 'undo', Icons.undo),
    CommandItem('redo', 'إعادة', 'redo', Icons.redo),
    CommandItem('save', 'حفظ المشروع', 'save', Icons.save_outlined),
    CommandItem('open', 'فتح مشروع', 'open', Icons.folder_open_outlined),
    CommandItem('perspective', 'عرض منظوري', 'perspective', Icons.pets),
    CommandItem('orthographic', 'عرض متعامد', 'orthographic', Icons.square_foot),
    CommandItem('top_view', 'عرض من أعلى', 'top_view', Icons.flip),
    CommandItem('front_view', 'عرض الواجهة', 'front_view', Icons.view_carousel),
    CommandItem('iso_view', 'عرض متساوية البعد', 'iso_view', Icons.view_compact),
  ];

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 150),
      vsync: this,
    );
    _scale = CurvedAnimation(parent: _controller, curve: Curves.easeOutBack);
    _filtered = _commands;
    _searchController.addListener(_filterCommands);
    if (widget.visible) _controller.forward();
  }

  @override
  void didUpdateWidget(covariant QuickCommandSearch oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.visible && !oldWidget.visible) {
      _controller.forward(from: 0);
      _searchController.clear();
      _filtered = _commands;
      _selectedIndex = 0;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        _focusNode.requestFocus();
      });
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    _searchController.dispose();
    _focusNode.dispose();
    super.dispose() {
    }
  }

  void _filterCommands() {
    final query = _searchController.text.toLowerCase();
    setState(() {
      _filtered = _commands.where((cmd) {
        return cmd.label.contains(query) ||
            cmd.id.contains(query) ||
            cmd.category.contains(query);
      }).toList();
      _selectedIndex = _selectedIndex.clamp(0, _filtered.length - 1);
    });
  }

  void _selectCommand(CommandItem cmd) {
    widget.onCommandSelected(cmd.id);
    widget.onDismiss();
  }

  @override
  Widget build(BuildContext context) {
    if (!widget.visible) return const SizedBox.shrink();

    return GestureDetector(
      onTap: widget.onDismiss,
      child: Container(
        color: const Color(0x80000000),
        child: Center(
          child: GestureDetector(
            onTap: () {}, // Absorb taps on the search box
            child: ScaleTransition(
              scale: _scale,
              child: Container(
                width: 420,
                constraints: BoxConstraints(
                  maxHeight: MediaQuery.of(context).size.height * 0.6,
                ),
                margin: const EdgeInsets.symmetric(horizontal: 24),
                decoration: BoxDecoration(
                  color: const Color(0xff0d2130),
                  borderRadius: BorderRadius.circular(16),
                  border: Border.all(color: const Color(0xff2a5260)),
                  boxShadow: const [
                    BoxShadow(
                      color: Color(0x60000000),
                      blurRadius: 24,
                      offset: Offset(0, 8),
                    ),
                  ],
                ),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    // Search input
                    Container(
                      padding: const EdgeInsets.all(12),
                      decoration: const BoxDecoration(
                        border: Border(
                          bottom: BorderSide(color: Color(0xff1d3547)),
                        ),
                      ),
                      child: Row(
                        children: [
                          const Icon(
                            Icons.search,
                            color: Color(0xff54e0d7),
                            size: 20,
                          ),
                          const SizedBox(width: 10),
                          Expanded(
                            child: TextField(
                              controller: _searchController,
                              focusNode: _focusNode,
                              style: const TextStyle(
                                color: Colors.white,
                                fontSize: 15,
                              ),
                              decoration: const InputDecoration(
                                hintText: 'اكتب اسم الأداة...',
                                hintStyle: TextStyle(
                                  color: Color(0xff668391),
                                ),
                                border: InputBorder.none,
                                contentPadding: EdgeInsets.zero,
                              ),
                              onSubmitted: (_) {
                                if (_filtered.isNotEmpty) {
                                  _selectCommand(_filtered[_selectedIndex]);
                                }
                              },
                            ),
                          ),
                          Container(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 6,
                              vertical: 3,
                            ),
                            decoration: BoxDecoration(
                              color: const Color(0xff183344),
                              borderRadius: BorderRadius.circular(4),
                              border: Border.all(
                                color: const Color(0xff2a5260),
                              ),
                            ),
                            child: const Text(
                              'ESC',
                              style: TextStyle(
                                color: Color(0xff7793a3),
                                fontSize: 10,
                                fontWeight: FontWeight.w600,
                              ),
                            ),
                          ),
                        ],
                      ),
                    ),
                    // Results
                    Flexible(
                      child: _filtered.isEmpty
                          ? const Padding(
                              padding: EdgeInsets.all(24),
                              child: Text(
                                'لا توجد نتائج',
                                style: TextStyle(
                                  color: Color(0xff668391),
                                  fontSize: 13,
                                ),
                              ),
                            )
                          : ListView.builder(
                              shrinkWrap: true,
                              padding: const EdgeInsets.symmetric(vertical: 4),
                              itemCount: _filtered.length,
                              itemBuilder: (context, index) {
                                final cmd = _filtered[index];
                                final selected = index == _selectedIndex;
                                return InkWell(
                                  onTap: () => _selectCommand(cmd),
                                  child: Container(
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 12,
                                      vertical: 8,
                                    ),
                                    color: selected
                                        ? const Color(0xff164b56).withOpacity(0.5)
                                        : null,
                                    child: Row(
                                      children: [
                                        Icon(
                                          cmd.icon,
                                          size: 18,
                                          color: selected
                                              ? const Color(0xff62eee2)
                                              : const Color(0xff7793a3),
                                        ),
                                        const SizedBox(width: 10),
                                        Expanded(
                                          child: Text(
                                            cmd.label,
                                            style: TextStyle(
                                              color: selected
                                                  ? const Color(0xffd5e1e7)
                                                  : const Color(0xffa7bdc9),
                                              fontSize: 13,
                                              fontWeight: selected
                                                  ? FontWeight.w600
                                                  : FontWeight.w400,
                                            ),
                                          ),
                                        ),
                                        Text(
                                          cmd.category,
                                          style: const TextStyle(
                                            color: Color(0xff546570),
                                            fontSize: 10,
                                          ),
                                        ),
                                      ],
                                    ),
                                  ),
                                );
                              },
                            ),
                    ),
                    // Footer hint
                    Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 12,
                        vertical: 8,
                      ),
                      decoration: const BoxDecoration(
                        border: Border(
                          top: BorderSide(color: Color(0xff1d3547)),
                        ),
                      ),
                      child: Row(
                        children: [
                          _KeyHint(icon: Icons.keyboard_arrow_up, label: 'تنقل'),
                          const SizedBox(width: 12),
                          _KeyHint(icon: Icons.keyboard_return, label: 'اختيار'),
                          const SizedBox(width: 12),
                          _KeyHint(icon: Icons.keyboard, label: 'إدخال'),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _KeyHint extends StatelessWidget {
  const _KeyHint({required this.icon, required this.label});
  final IconData icon;
  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(icon, size: 12, color: const Color(0xff668391)),
        const SizedBox(width: 3),
        Text(
          label,
          style: const TextStyle(
            color: Color(0xff668391),
            fontSize: 10,
          ),
        ),
      ],
    );
  }
}

class CommandItem {
  const CommandItem(this.id, this.label, this.category, this.icon);
  final String id;
  final String label;
  final String category;
  final IconData icon;
}
