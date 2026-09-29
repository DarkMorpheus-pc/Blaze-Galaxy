#include "translator.hpp"

#include <qcoreapplication.h>
#include <qcoreevent.h>
#include <qdirlisting.h>
#include <qendian.h>
#include <qfile.h>
#include <qlocale.h>
#include <qloggingcategory.h>
#include <qtranslator.h>
#include <qqmlengine.h>

#include <cstring>

#include "config/rootnodes.hpp"
#include "util/i18n.hpp"

namespace {

Q_LOGGING_CATEGORY(lcI18n, "caelestia.i18n", QtInfoMsg)

} // namespace

namespace caelestia::i18n {

using Qt::StringLiterals::operator""_s;

namespace {

constexpr quint32 k_magic = 0x950412de;
constexpr quint32 k_magicSwapped = 0xde120495;
constexpr qsizetype k_headerSize = 28;

// .mo header field offsets
constexpr qsizetype k_countOffset = 8;
constexpr qsizetype k_origsOffset = 12;
constexpr qsizetype k_transOffset = 16;

// Table entries are a uint32 length followed by a uint32 offset
constexpr qsizetype k_entrySize = 8;
constexpr qsizetype k_entryOffsetField = 4;

QString resourceDir() {
    static const QString k_s = u":/qt/qml/Caelestia/I18n/"_s;
    return k_s;
}

// Substitutes the count into a plural form. `%n` is plain, `%Ln` is localised
void substitutePercentN(QString& text, int n) {
    qsizetype pos = 0;
    while ((pos = text.indexOf(u'%', pos)) >= 0) {
        auto len = 1;
        const auto localised = pos + len < text.size() && text.at(pos + len) == u'L';
        if (localised)
            ++len;

        if (pos + len >= text.size() || text.at(pos + len) != u'n') {
            pos += len;
            continue;
        }

        const auto count = localised ? QLocale().toString(n) : QString::number(n);
        text.replace(pos, len + 1, count);
        pos += count.size();
    }
}

class CaelestiaQTranslator : public QTranslator {
public:
    explicit CaelestiaQTranslator(Translator* t) : QTranslator(t), m_translator(t) {}

    // The catalog is gettext-backed, not QTranslator's internal .qm data.
    bool isEmpty() const override { return false; }

    QString translate(const char* context, const char* sourceText,
                      const char* disambiguation = nullptr, int n = -1) const override {
        Q_UNUSED(disambiguation);
        if (!m_translator || !sourceText || !*sourceText)
            return QString();

        // 1. If English is selected, ALWAYS return sourceText directly!
        // This locks the English translation and prevents Qt from falling back to
        // any system locale / Qt translation that might turn it back to Turkish.
        if (m_translator->language() == u"en"_s)
            return QString::fromUtf8(sourceText);

        if (m_translator->catalogCount() == 0)
            return QString();

        const QString text = QString::fromUtf8(sourceText);
        const QString ctx = context ? QString::fromUtf8(context) : QString();

        bool found = false;
        QString res;

        // 2. Catalogs are gettext-based and mostly context-free.
        // First check with context if provided:
        if (!ctx.isEmpty()) {
            res = (n >= 0) ? m_translator->lookupPlural(text, text, n, ctx, &found)
                           : m_translator->lookupSingle(text, ctx, &found);
        }

        // If not found with context, query without context (primary for gettext catalogues):
        if (!found) {
            res = (n >= 0) ? m_translator->lookupPlural(text, text, n, {}, &found)
                           : m_translator->lookupSingle(text, {}, &found);
        }

        if (found) {
            return res;
        }

        return QString();
    }
private:
    Translator* m_translator = nullptr;
};

} // namespace

Translator::Translator(QObject* parent)
    : QObject(parent)
    , m_supportedLanguages(findSupportedLangs()) {
    auto* const general = config::ConfigSingleton::instance()->general();
    QObject::connect(general, &config::GeneralConfig::languageChanged, this, [this, general]() {
        setLanguage(resolveLanguage(general->language()));
    });

    m_language = resolveLanguage(general->language());
    loadTranslations();

    if (QCoreApplication::instance()) {
        QCoreApplication::installTranslator(new CaelestiaQTranslator(this));
    }
}

bool Translator::trsChangedFlag() {
    return false;
}

QStringList Translator::supportedLanguages() const {
    return m_supportedLanguages;
}

QString Translator::language() const {
    return m_language;
}

void Translator::setLanguageQml(const QString& language) {
    // The connection in the ctor will update the lang in this
    config::ConfigSingleton::instance()->general()->set_language(language);
}

QString Translator::_tr(const QString& text, const QString& context, bool markedOnly) const {
    if (text.isEmpty())
        return text;

    if (util::i18n::isMarked(text)) {
        if (!context.isEmpty())
            qCWarning(lcI18n) << "Attempted to translate a marked string with context. "
                                 "Context should be baked in when marking, ignoring context.";

        const auto marked = util::i18n::parseMarked(text);
        auto result = marked.n < 0 ? translate(marked.text, marked.context)
                                   : translatePlural(marked.text, marked.plural, marked.n, marked.context);
        for (const auto& arg : marked.args)
            result = result.arg(util::i18n::isMarked(arg) ? _tr(arg, {}, true) : arg);
        return result;
    }

    if (markedOnly)
        return text; // Don't translate unmarked strings when markedOnly

    return translate(text, context);
}

QString Translator::_trN(const QString& text, const QString& plural, int n, const QString& context) const {
    if (util::i18n::isMarked(text)) {
        qCWarning(lcI18n) << "Attempted to translate a marked string with plural forms. "
                             "Plural forms should be baked in when marking, ignoring plural forms.";
        return _tr(text, context, false);
    }

    if (text.isEmpty())
        return text;

    return translatePlural(text, plural, n, context);
}

QString Translator::mark(const QString& text, const QStringList& args) {
    return util::i18n::detail::doMark(text, {}, -1, {}, args);
}

QString Translator::markCtx(const QString& text, const QString& context, const QStringList& args) {
    return util::i18n::detail::doMark(text, {}, -1, context, args);
}

QString Translator::markN(const QString& text, const QString& plural, int n, const QStringList& args) {
    return util::i18n::detail::doMark(text, plural, n, {}, args);
}

QString Translator::markCtxN(
    const QString& text, const QString& plural, int n, const QString& context, const QStringList& args) {
    return util::i18n::detail::doMark(text, plural, n, context, args);
}

QStringList Translator::findSupportedLangs() {
    QStringList langs { u"en"_s };
    const QDirListing listing(resourceDir(), { u"*.mo"_s }, QDirListing::IteratorFlag::FilesOnly);
    for (const auto& f : listing)
        langs << f.completeBaseName();

    const QDirListing fsListing(u"/usr/share/caelestia/translations"_s, { u"*.mo"_s }, QDirListing::IteratorFlag::FilesOnly);
    for (const auto& f : fsListing) {
        const auto base = f.completeBaseName();
        if (!langs.contains(base))
            langs << base;
    }
    return langs;
}

void Translator::loadTranslations() {
    m_catalog.clear();
    m_count = 0;

    if (m_language.isEmpty() || m_language == u"en"_s)
        return;

    QFile file(resourceDir() + m_language + u".mo"_s);
    if (!file.exists()) {
        file.setFileName(u"/usr/share/caelestia/translations/"_s + m_language + u".mo"_s);
    }
    if (!file.open(QIODevice::ReadOnly)) {
        qCWarning(lcI18n) << "Failed to open catalog for" << m_language;
        return;
    }

    auto data = file.readAll();
    if (data.size() < k_headerSize) {
        qCWarning(lcI18n) << "Truncated catalog for" << m_language;
        return;
    }

    // Magic read as big endian to find the file's real byte order
    const auto magic = qFromBigEndian<quint32>(data.constData());
    if (magic != k_magic && magic != k_magicSwapped) {
        qCWarning(lcI18n) << "Bad magic in catalog for" << m_language;
        return;
    }

    m_littleEndian = magic == k_magicSwapped;
    m_catalog = std::move(data);

    const auto count = readU32(k_countOffset);
    const auto origs = readU32(k_origsOffset);
    const auto trans = readU32(k_transOffset);

    // Validate tables
    const auto tableSize = k_entrySize * static_cast<qsizetype>(count);
    if (static_cast<qsizetype>(origs) + tableSize > m_catalog.size() ||
        static_cast<qsizetype>(trans) + tableSize > m_catalog.size()) {
        qCWarning(lcI18n) << "Corrupt string tables in catalog for" << m_language;
        m_catalog.clear();
        return;
    }

    m_count = count;
    m_origs = origs;
    m_trans = trans;

    // The metadata entry keyed by the empty msgid carries the Plural-Forms rule
    if (!m_plurals.parse(lookupRaw("")))
        qCDebug(lcI18n) << "No usable Plural-Forms for" << m_language << "- assuming the default rule";

    qCDebug(lcI18n) << "Loaded" << m_count << "messages for" << m_language;
}

quint32 Translator::readU32(qsizetype offset) const {
    const auto* raw = m_catalog.constData();
    return m_littleEndian ? qFromLittleEndian<quint32>(raw + offset) : qFromBigEndian<quint32>(raw + offset);
}

QByteArray Translator::catalogKey(const QString& text, const QString& context) {
    if (context.isEmpty())
        return text.toUtf8();
    return context.toUtf8() + util::i18n::k_contextSep.toLatin1() + text.toUtf8();
}

QString Translator::segment(QByteArrayView blob, quint32 index) {
    for (quint32 i = 0; i < index; ++i) {
        const auto nul = blob.indexOf('\0');
        if (nul < 0)
            return {}; // Catalog has fewer forms than the rule asked for
        blob = blob.sliced(nul + 1);
    }

    const auto nul = blob.indexOf('\0');
    return QString::fromUtf8(nul < 0 ? blob : blob.first(nul));
}

QByteArrayView Translator::lookupRaw(QByteArrayView key) const {
    const auto* raw = m_catalog.constData();

    // Entries are sorted against msgid, so we can use a binary search
    quint32 lo = 0;
    quint32 hi = m_count;
    while (lo < hi) {
        const auto probe = lo + (hi - lo) / 2;
        const auto entry = static_cast<qsizetype>(m_origs) + k_entrySize * probe;
        const auto len = static_cast<qsizetype>(readU32(entry));
        const auto off = static_cast<qsizetype>(readU32(entry + k_entryOffsetField));
        if (off + len > m_catalog.size())
            return {};

        // Plural entries store `msgid \0 msgid_plural`, only the msgid is part of the key
        const QByteArrayView orig(raw + off, len);
        const auto nul = orig.indexOf('\0');
        const auto msgid = nul < 0 ? orig : orig.first(nul);

        auto cmp = std::memcmp(key.constData(), msgid.constData(), static_cast<size_t>(qMin(key.size(), msgid.size())));
        if (cmp == 0 && key.size() != msgid.size())
            cmp = key.size() < msgid.size() ? -1 : 1;

        if (cmp == 0) {
            const auto hit = static_cast<qsizetype>(m_trans) + k_entrySize * probe;
            const auto tlen = static_cast<qsizetype>(readU32(hit));
            const auto toff = static_cast<qsizetype>(readU32(hit + k_entryOffsetField));
            if (toff + tlen > m_catalog.size())
                return {};
            return { raw + toff, tlen };
        }

        if (cmp < 0)
            hi = probe;
        else
            lo = probe + 1;
    }

    return {};
}

QString Translator::lookup(QByteArrayView key, quint32 index) const {
    const auto blob = lookupRaw(key);
    return blob.isNull() ? QString() : segment(blob, index);
}

QString Translator::lookupSingle(const QString& text, const QString& context, bool* found) const {
    if (found)
        *found = false;
    if (m_count == 0 || text.isEmpty())
        return text;

    const auto blob = lookupRaw(catalogKey(text, context));
    if (!blob.isNull()) {
        if (found)
            *found = true;
        return segment(blob, 0);
    }
    return text;
}

QString Translator::lookupPlural(
    const QString& text, const QString& plural, int n, const QString& context, bool* found) const {
    if (found)
        *found = false;
    auto result = n == 1 ? text : plural;

    if (m_count > 0 && !text.isEmpty()) {
        const auto blob = lookupRaw(catalogKey(text, context));
        if (!blob.isNull()) {
            if (found)
                *found = true;
            result = segment(blob, m_plurals.evaluate(n));
        }
    }

    substitutePercentN(result, n);
    return result;
}

QString Translator::translate(const QString& text, const QString& context) const {
    return lookupSingle(text, context);
}

QString Translator::translatePlural(const QString& text, const QString& plural, int n, const QString& context) const {
    return lookupPlural(text, plural, n, context);
}

QString Translator::langForLocale() const {
    const auto langs = QLocale::system().uiLanguages(QLocale::TagSeparator::Underscore);
    for (const auto& lang : langs) {
        if (m_supportedLanguages.contains(lang))
            return lang;
        if (lang.startsWith(u"tr"_s) && m_supportedLanguages.contains(u"tr"_s))
            return u"tr"_s;
    }

    const auto name = QLocale::system().name();
    if (name.startsWith(u"tr"_s) && m_supportedLanguages.contains(u"tr"_s))
        return u"tr"_s;

    qCDebug(lcI18n) << "No catalog for any of the system UI languages";
    return {};
}

QString Translator::resolveLanguage(const QString& language) const {
    if (language.isEmpty())
        return langForLocale();

    if (language == u"en"_s)
        return u"en"_s;

    const auto normalised = QString(language).replace(u'-', u'_');
    if (m_supportedLanguages.contains(normalised))
        return normalised;
    const auto base = normalised.section(u'_', 0, 0);
    if (m_supportedLanguages.contains(base))
        return base;

    if (!m_supportedLanguages.contains(language)) {
        qCWarning(lcI18n) << "Unsupported language" << language << "- falling back to the system locale";
        return langForLocale();
    }

    return language;
}

void Translator::setLanguage(const QString& language) {
    if (m_language == language)
        return;

    m_language = language;
    loadTranslations(); // Load before emitting cause trsChangedFlag reuses the signal
    emit languageChanged();
    // LanguageChange alone is not sufficient for every QQmlEngine host.
    if (auto* engine = qmlEngine(this))
        engine->retranslate();

    if (auto* app = QCoreApplication::instance()) {
        QCoreApplication::postEvent(app, new QEvent(QEvent::LanguageChange));
    }
}

} // namespace caelestia::i18n
