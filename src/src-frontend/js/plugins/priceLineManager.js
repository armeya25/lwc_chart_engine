/* Price Line Manager */
export const PriceLineManager = {
    _lines: new Map(), // lineId -> { seriesId, lineObj }

    create: function (seriesId, lineId, options) {
        const series = window.seriesMap.get(seriesId);
        if (series) {
            const lineObj = series.createPriceLine(options);
            this._lines.set(lineId, { seriesId, lineObj });
        }
    },

    remove: function (lineId) {
        const record = this._lines.get(lineId);
        if (record) {
            const series = window.seriesMap.get(record.seriesId);
            if (series) {
                series.removePriceLine(record.lineObj);
            }
            this._lines.delete(lineId);
        }
    },

    update: function (lineId, options) {
        const record = this._lines.get(lineId);
        if (record) {
            record.lineObj.applyOptions(options);
        }
    }
};

window.PriceLineManager = PriceLineManager;
