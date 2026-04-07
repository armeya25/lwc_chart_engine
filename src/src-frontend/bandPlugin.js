class BandSeriesPaneRenderer {
    constructor(data, options) {
        this.data = data;
        this.options = options;
    }
    drawBackground(target) {
        if (!this.data || this.data.length === 0) return;
        target.useBitmapCoordinateSpace((scope) => {
            const ctx = scope.context;
            ctx.scale(scope.horizontalPixelRatio, scope.verticalPixelRatio);
            ctx.fillStyle = this.options.color;

            const points = this.data;

            // Filter to only points within (or very near) the visible canvas area
            // timeToCoordinate returns non-null even for off-screen bars, so we clamp
            const canvasW = scope.bitmapSize.width / scope.horizontalPixelRatio;
            const canvasH = scope.bitmapSize.height / scope.verticalPixelRatio;
            const margin = 50; // small px margin outside canvas to allow edge polygons

            const visiblePoints = points.filter(p =>
                p.top !== null && p.bottom !== null &&
                p.x > -margin && p.x < canvasW + margin
            );

            if (visiblePoints.length === 0) return;

            // Estimate bar half-width from spacing of the first visible points
            let barHalfWidth = 4;
            if (visiblePoints.length > 1) {
                let spacingSum = 0, spacingCount = 0;
                for (let i = 1; i < Math.min(visiblePoints.length, 20); i++) {
                    const gap = Math.abs(visiblePoints[i].x - visiblePoints[i - 1].x);
                    if (gap > 0.5 && gap < 200) {
                        spacingSum += gap;
                        spacingCount++;
                    }
                }
                if (spacingCount > 0) barHalfWidth = (spacingSum / spacingCount) / 2 + 0.5;
            }

            // --- Group visiblePoints into contiguous segments ---
            // A gap exists when consecutive x values differ by more than 3x typical bar spacing
            const gapThreshold = barHalfWidth * 2 * 3;
            const segments = [];
            let seg = [visiblePoints[0]];
            for (let i = 1; i < visiblePoints.length; i++) {
                if (Math.abs(visiblePoints[i].x - visiblePoints[i - 1].x) > gapThreshold) {
                    segments.push(seg);
                    seg = [];
                }
                seg.push(visiblePoints[i]);
            }
            segments.push(seg);

            // Draw each segment as a closed polygon
            for (const s of segments) {
                if (s.length === 0) continue;
                ctx.beginPath();
                ctx.moveTo(s[0].x, s[0].top);
                for (let i = 1; i < s.length; i++) ctx.lineTo(s[i].x, s[i].top);
                for (let i = s.length - 1; i >= 0; i--) ctx.lineTo(s[i].x, s[i].bottom);
                ctx.closePath();
                ctx.fill();
            }
        });
    }
    draw() {}
}

class BandSeriesPaneView {
    constructor(source) {
        this.source = source;
        this._views = [];
    }
    update() {
        if (!this.source.chart || !this.source.series) return;
        const timeScale = this.source.chart.timeScale();
        const mainSeries = this.source.series;
        this._views = [];

        for (const pt of this.source.data) {
            const x = timeScale.timeToCoordinate(pt.time);
            if (x === null) continue;

            const top = (pt.top !== undefined && pt.top !== null) ? mainSeries.priceToCoordinate(pt.top) : null;
            const bottom = (pt.bottom !== undefined && pt.bottom !== null) ? mainSeries.priceToCoordinate(pt.bottom) : null;

            if (top !== null && bottom !== null) {
                this._views.push({ x, top, bottom });
            }
        }
    }
    renderer() {
        return new BandSeriesPaneRenderer(this._views, this.source.options);
    }
}

class BandSeriesPrimitive {
    constructor(options) {
        this.options = options;
        this.data = [];
        this._paneViews = [new BandSeriesPaneView(this)];
    }
    attached({ chart, series, requestUpdate }) {
        this.chart = chart;
        this.series = series;
        this.requestUpdate = requestUpdate;
    }
    detached() {
        this.chart = null;
        this.series = null;
    }
    updateAllViews() {
        this._paneViews.forEach(v => v.update());
    }
    paneViews() { return this._paneViews; }
    setData(data) {
        this.data = data;
        if (this.requestUpdate) this.requestUpdate();
    }
}
window.BandSeriesPrimitive = BandSeriesPrimitive;
