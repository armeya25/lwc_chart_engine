/* Marker Manager */
const MarkerManager = {
    _markers: new Map(), // (chartId + seriesId) -> array of marker objects

    _getKey: function(chartId, seriesId) {
        return `${chartId || 'chart-0'}_${seriesId}`;
    },

    addMarker: function (seriesId, markerData, chartId) {
        const key = this._getKey(chartId, seriesId);
        if (!this._markers.has(key)) this._markers.set(key, []);
        const seriesMarkers = this._markers.get(key);
        seriesMarkers.push(markerData);
        this.applyMarkers(seriesId, chartId);
    },

    addMarkersBulk: function (seriesId, markersArray, chartId) {
        const key = this._getKey(chartId, seriesId);
        // Correct approach: Replace the entire list
        this._markers.set(key, markersArray);
        this.applyMarkers(seriesId, chartId);
    },

    removeMarker: function (seriesId, markerId, chartId) {
        const key = this._getKey(chartId, seriesId);
        if (this._markers.has(key)) {
            const seriesMarkers = this._markers.get(key).filter(m => m.id !== markerId);
            this._markers.set(key, seriesMarkers);
            this.applyMarkers(seriesId, chartId);
        }
    },

    clearAll: function () {
        this._markers.forEach((markers, key) => {
            const seriesId = key.split('_')[1];
            const chartId = key.split('_')[0];
            this._markers.set(key, []);
            this.applyMarkers(seriesId, chartId);
        });
        this._markers.clear();
    },

    updateMarker: function (seriesId, markerId, changes, chartId) {
        const key = this._getKey(chartId, seriesId);
        if (this._markers.has(key)) {
            const seriesMarkers = this._markers.get(key);
            const marker = seriesMarkers.find(m => m.id === markerId);
            if (marker) {
                Object.assign(marker, changes);
                this.applyMarkers(seriesId, chartId);
            }
        }
    },

    applyMarkers: function (seriesId, chartId) {
        if (!this._renderCache) this._renderCache = new Set();
        const cacheKey = `${chartId || 'chart-0'}_${seriesId}`;
        this._renderCache.add(cacheKey);

        if (this._rafId) return;
        this._rafId = requestAnimationFrame(() => {
            this._renderCache.forEach(key => {
                const parts = key.split('_');
                const [cId, sId] = [parts[0], parts[1]];
                const series = window.seriesMap.get(sId);
                if (series) {
                    const markers = (this._markers.get(key) || []).slice();
                    markers.sort((a, b) => a.time - b.time);
                    series.setMarkers(markers);
                }
            });
            this._renderCache.clear();
            this._rafId = null;
        });
    }
};

window.MarkerManager = MarkerManager;
