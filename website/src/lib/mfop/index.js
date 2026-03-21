// Locale-aware MFOP spec loader.
// Each locale file exports { mfopMeta, mfopSections } with translated content.

const SUPPORTED = [
  'en-US', 'es-ES', 'fr-FR', 'de-DE', 'pt-BR',
  'ja-JP', 'zh-CN', 'ko-KR', 'hi-IN', 'ar-SA',
];

export async function getMfopSpec(locale) {
  const target = SUPPORTED.includes(locale) ? locale : 'en-US';
  try {
    const mod = await import(`./${target}.js`);
    return { mfopMeta: mod.mfopMeta, mfopSections: mod.mfopSections };
  } catch {
    // Fall back to English if translation file is missing
    const mod = await import('./en-US.js');
    return { mfopMeta: mod.mfopMeta, mfopSections: mod.mfopSections };
  }
}
