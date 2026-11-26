import React, { useEffect } from 'react';
import { EFFORTS } from '../i18n';

export function attachLightDismissFallback(dialog) {
  if (!dialog || 'closedBy' in HTMLDialogElement.prototype) return () => {};

  let pressedOutside = false;
  const isOutside = (e) => {
    const r = dialog.getBoundingClientRect();
    return e.clientX < r.left || e.clientX > r.right || e.clientY < r.top || e.clientY > r.bottom;
  };
  const onDown = (e) => {
    pressedOutside = isOutside(e);
  };
  const onUp = (e) => {
    if (pressedOutside && isOutside(e)) dialog.close();
  };
  dialog.addEventListener('pointerdown', onDown);
  dialog.addEventListener('pointerup', onUp);
  return () => {
    dialog.removeEventListener('pointerdown', onDown);
    dialog.removeEventListener('pointerup', onUp);
  };
}

export default function MatchDialogs({
  menuDialogRef,
  rulesDialogRef,
  t,
  draftMode,
  setDraftMode,
  draftSide,
  setDraftSide,
  draftKomi,
  setDraftKomi,
  effort,
  onEffortSelect,
  showAnchors,
  setShowAnchors,
  soundEnabled,
  setSoundEnabled,
  lang,
  setLang,
  matchInProgress,
  onStartMatch,
}) {
  useEffect(() => {
    const cleanMenu = attachLightDismissFallback(menuDialogRef.current);
    const cleanRules = attachLightDismissFallback(rulesDialogRef.current);
    return () => {
      cleanMenu();
      cleanRules();
    };
  }, [menuDialogRef, rulesDialogRef]);

  return (
    <>
      <dialog
        ref={menuDialogRef}
        className="modal-dialog"
        closedby="any"
        aria-labelledby="menu-title"
        data-testid="menu-dialog"
      >
        <div className="dialog-header">
          <h1 id="menu-title">{t.menuTitle}</h1>
          <button
            type="button"
            className="btn btn-subtle icon-close"
            aria-label={t.close}
            onClick={() => menuDialogRef.current?.close()}
          >
            ×
          </button>
        </div>

        <div className="menu-sections">
          <div className="menu-field">
            <span className="menu-label">{t.modeLabel}</span>
            <div className="menu-segmented" role="group" aria-label={t.modeLabel}>
              <button
                type="button"
                className="segment-btn"
                aria-pressed={draftMode === 'vs_ai'}
                data-testid="mode-vs-ai"
                onClick={() => setDraftMode('vs_ai')}
              >
                {t.modeVsAi}
              </button>
              <button
                type="button"
                className="segment-btn"
                aria-pressed={draftMode === '2p'}
                data-testid="mode-2p"
                onClick={() => setDraftMode('2p')}
              >
                {t.mode2p}
              </button>
            </div>
          </div>

          {draftMode === 'vs_ai' && (
            <div className="menu-field">
              <span className="menu-label">{t.playAsLabel}</span>
              <div className="menu-segmented" role="group" aria-label={t.playAsLabel}>
                <button
                  type="button"
                  className="segment-btn"
                  aria-pressed={draftSide === 1}
                  data-testid="side-p1"
                  onClick={() => setDraftSide(1)}
                >
                  <span className="swatch swatch-p1" aria-hidden="true" />
                  {t.playAsP1}
                </button>
                <button
                  type="button"
                  className="segment-btn"
                  aria-pressed={draftSide === 2}
                  data-testid="side-p2"
                  onClick={() => setDraftSide(2)}
                >
                  <span className="swatch swatch-p2" aria-hidden="true" />
                  {t.playAsP2(draftKomi)}
                </button>
              </div>
            </div>
          )}

          {draftMode === 'vs_ai' && (
            <div className="menu-field">
              <span className="menu-label">{t.effortLabel}</span>
              <div className="menu-segmented quad" role="group" aria-label={t.effortLabel}>
                {EFFORTS.map((tier) => (
                  <button
                    key={tier}
                    type="button"
                    className="segment-btn"
                    aria-pressed={effort === tier}
                    data-testid={`menu-effort-${tier}`}
                    onClick={() => onEffortSelect(tier)}
                  >
                    {t.effortLabels[tier]}
                  </button>
                ))}
              </div>
              <p className="effort-hint">{t.effortHints[effort]}</p>
            </div>
          )}

          <div className="menu-field">
            <span className="menu-label">{t.ruleSetLabel}</span>
            <div className="menu-segmented" role="group" aria-label={t.ruleSetLabel}>
              <button
                type="button"
                className="segment-btn"
                aria-pressed={draftKomi === 18}
                data-testid="komi-balanced"
                onClick={() => setDraftKomi(18)}
              >
                {t.ruleBalanced}
              </button>
              <button
                type="button"
                className="segment-btn"
                aria-pressed={draftKomi === 0}
                data-testid="komi-classic"
                onClick={() => setDraftKomi(0)}
              >
                {t.ruleClassic}
              </button>
            </div>
          </div>

          <div className="menu-field">
            <span className="menu-label">{t.prefsLabel}</span>
            <div className="pref-list">
              <label className="pref-row" htmlFor="pref-anchors">
                <span>{t.prefAnchors}</span>
                <input
                  id="pref-anchors"
                  type="checkbox"
                  checked={showAnchors}
                  data-testid="pref-anchors"
                  onChange={(e) => setShowAnchors(e.target.checked)}
                />
              </label>
              <label className="pref-row" htmlFor="pref-sound">
                <span>{t.prefSound}</span>
                <input
                  id="pref-sound"
                  type="checkbox"
                  checked={soundEnabled}
                  data-testid="pref-sound"
                  onChange={(e) => setSoundEnabled(e.target.checked)}
                />
              </label>
              <div className="pref-lang-row">
                <span>{t.prefLang}</span>
                <div className="segmented-pill" role="group" aria-label={t.prefLang}>
                  <button
                    type="button"
                    className="segment-btn"
                    aria-pressed={lang === 'en'}
                    data-testid="lang-en"
                    onClick={() => setLang('en')}
                  >
                    EN
                  </button>
                  <button
                    type="button"
                    className="segment-btn"
                    aria-pressed={lang === 'ru'}
                    data-testid="lang-ru"
                    onClick={() => setLang('ru')}
                  >
                    RU
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div className="dialog-footer">
          {matchInProgress && (
            <button
              type="button"
              className="btn"
              data-testid="resume-btn"
              onClick={() => menuDialogRef.current?.close()}
            >
              {t.resume}
            </button>
          )}
          <button
            type="button"
            className="btn btn-primary"
            data-testid="start-match-btn"
            onClick={() => {
              menuDialogRef.current?.close();
              onStartMatch(draftMode, draftSide, draftKomi);
            }}
          >
            {t.startMatch}
          </button>
        </div>
      </dialog>

      <dialog
        ref={rulesDialogRef}
        className="modal-dialog"
        closedby="any"
        aria-labelledby="rules-title"
        data-testid="rules-dialog"
      >
        <div className="dialog-header">
          <h1 id="rules-title">{t.rulesTitle}</h1>
          <button
            type="button"
            className="btn btn-subtle icon-close"
            aria-label={t.close}
            onClick={() => rulesDialogRef.current?.close()}
          >
            ×
          </button>
        </div>
        <ol className="rules-list">
          {t.rulesItems.map((item, idx) => (
            <li key={idx}>
              <strong>{item.head}</strong> {item.body}
            </li>
          ))}
        </ol>
        <div className="dialog-footer">
          <form method="dialog">
            <button type="submit" className="btn btn-primary" data-testid="rules-close-btn" autoFocus>
              {t.close}
            </button>
          </form>
        </div>
      </dialog>
    </>
  );
}
