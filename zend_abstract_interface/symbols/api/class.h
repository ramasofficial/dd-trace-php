#ifndef HAVE_ZAI_SYMBOLS_API_CLASS_H
#define HAVE_ZAI_SYMBOLS_API_CLASS_H
// clang-format off
static inline zend_class_entry *zai_symbol_lookup_class(
                                    zai_symbol_scope_t scope_type, void *scope,
                                    zai_string_view *name	) {
    return (zend_class_entry *)zai_symbol_lookup(ZAI_SYMBOL_TYPE_CLASS, scope_type, scope, name	);
}

static inline zend_class_entry *zai_symbol_lookup_class_literal(const char *cn, size_t cnl	) {
    zai_string_view vcn =
        (zai_string_view){cnl, cn};

    return zai_symbol_lookup_class(ZAI_SYMBOL_SCOPE_GLOBAL, NULL, &vcn	);
}

static inline zend_class_entry *zai_symbol_lookup_class_literal_ns(const char *ns, size_t nsl,
                                                                   const char *cn, size_t cnl	) {
    zai_string_view vns =
        (zai_string_view){nsl, ns};
    zai_string_view vcn =
        (zai_string_view){cnl, cn};

    return zai_symbol_lookup_class(ZAI_SYMBOL_SCOPE_NAMESPACE, &vns, &vcn	);
}
// clang-format on
#endif
