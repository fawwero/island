// i18n.ts — локализация RU/EN (ТЗ §3 «Локализация», §5.6).
// Язык берётся из настройки general.language; переключение без перезапуска.

import i18n from "i18next";
import { initReactI18next } from "react-i18next";

export const resources = {
  ru: {
    translation: {
      app: { name: "Island" },
      island: {
        idleTooltip: "Остров активен",
      },
      errors: {
        db: "Ошибка базы данных",
        secret: "Ошибка хранилища секретов",
        invalidValue: "Недопустимое значение настройки",
        secretName: "Недопустимое имя секрета",
        aiNoKey: "Не задан API-ключ ИИ. Укажите его в Настройках → ИИ.",
        aiNoNetwork: "Нет соединения с сервером ИИ.",
        aiRateLimit: "Превышен лимит запросов. Попробуйте позже.",
        aiModelUnavailable: "Модель недоступна, выбрана запасная.",
        ocrNoLanguage: "Языковой пакет OCR не установлен (нужен RU или EN).",
        captureProtected: "Контент защищён от захвата экрана.",
        hotkeyConflict: "Хоткей занят другим приложением. Переназначьте его.",
      },
      tray: {
        show: "Показать/Скрыть острова",
        pauseHotkeys: "Пауза хоткеев",
        pauseClipboard: "Пауза записи буфера",
        settings: "Настройки",
        quit: "Выход",
      },
    },
  },
  en: {
    translation: {
      app: { name: "Island" },
      island: { idleTooltip: "Island is active" },
      errors: {
        db: "Database error",
        secret: "Secret storage error",
        invalidValue: "Invalid setting value",
        secretName: "Invalid secret name",
        aiNoKey: "AI API key is not set. Configure it in Settings → AI.",
        aiNoNetwork: "Cannot reach the AI server.",
        aiRateLimit: "Rate limit exceeded. Try again later.",
        aiModelUnavailable: "Model unavailable, fallback selected.",
        ocrNoLanguage: "OCR language pack missing (RU or EN required).",
        captureProtected: "Content is protected from screen capture.",
        hotkeyConflict: "Hotkey is taken by another app. Reassign it.",
      },
      tray: {
        show: "Show/Hide islands",
        pauseHotkeys: "Pause hotkeys",
        pauseClipboard: "Pause clipboard recording",
        settings: "Settings",
        quit: "Quit",
      },
    },
  },
} as const;

export type Language = keyof typeof resources;

export async function setupI18n(initial: Language = "ru") {
  await i18n.use(initReactI18next).init({
    resources,
    lng: initial,
    fallbackLng: "en",
    interpolation: { escapeValue: false },
  });
  return i18n;
}

export function applyLanguage(lang: Language) {
  if (i18n.isInitialized && i18n.language !== lang) {
    void i18n.changeLanguage(lang);
  }
}

export default i18n;
