/**
 * Entry Point for LWC Chart Engine
 * Bundles all core logic and plugins using ESM
 */

import { SyncManager } from './core.js';
import { 
    showNotification, 
    initCharts, 
    setupToolbar, 
    toggleLegend, 
    addLegendItem, 
    toggleTrendInfo, 
    scrollToRealTime, 
    changeLayout, 
    createLayout, 
    updatePositionsUI 
} from './ui.js';
import { 
    CommandQueue, 
    handleCommand, 
    hideLoader, 
    CommandHandlers 
} from './commands.js';

import { MarkerManager } from './plugins/markerManager.js';
import { PriceLineManager } from './plugins/priceLineManager.js';
import { BoxManager } from './plugins/boxManager.js';
import { PositionToolManager } from './plugins/positionToolManager.js';
import { LineToolManager } from './plugins/lineToolManager.js';
import { BandSeriesPrimitive } from './plugins/bandPlugin.js';

// --- Attach to Window for Backend/Bridge Compatibility ---
window.SyncManager = SyncManager;
window.showNotification = showNotification;
window.initCharts = initCharts;
window.setupToolbar = setupToolbar;
window.toggleLegend = toggleLegend;
window.addLegendItem = addLegendItem;
window.toggleTrendInfo = toggleTrendInfo;
window.scrollToRealTime = scrollToRealTime;
window.changeLayout = changeLayout;
window.createLayout = createLayout;
window.updatePositionsUI = updatePositionsUI;

window.CommandQueue = CommandQueue;
window.handleCommand = handleCommand;
window.hideLoader = hideLoader;
window.CommandHandlers = CommandHandlers;

window.MarkerManager = MarkerManager;
window.PriceLineManager = PriceLineManager;
window.BoxManager = BoxManager;
window.PositionToolManager = PositionToolManager;
window.LineToolManager = LineToolManager;
window.BandSeriesPrimitive = BandSeriesPrimitive;

// --- Final Initialization ---
try {
    initCharts();
    setupToolbar();
    
    const loadingText = document.querySelector('.loading-text');
    if (loadingText) loadingText.innerText = "Waiting for Backend...";
    
    // Notify Backend (Hybrid support for Tauri/WebView)
    if (window.__TAURI__) {
        window.isReady = true;
        const invoke = window.__TAURI__.core ? window.__TAURI__.core.invoke : window.__TAURI__.invoke;
        if (invoke) {
            invoke('frontend_ready')
                .then(() => { if (hideLoader) hideLoader(); })
                .catch(e => console.error("Frontend: Tauri mark_ready failed", e));
        }
        const listen = window.__TAURI__.event ? window.__TAURI__.event.listen : window.__TAURI__.listen;
        if (listen) {
            listen('command', (event) => {
                handleCommand(event.payload);
            }).catch(e => console.error("Frontend: listen failed", e));
        }
    } else if (window.pywebview && window.pywebview.api) {
        window.isReady = true;
        setTimeout(() => {
            window.pywebview.api.mark_ready();
        }, 100);
    } else {
        window.addEventListener('pywebviewready', () => {
            window.isReady = true;
            setTimeout(() => {
                window.pywebview.api.mark_ready()
                    .catch(e => console.error("Frontend: mark_ready failed", e));
            }, 100);
        });
    }
} catch (e) {
    console.error("Initialization Error:", e);
}
