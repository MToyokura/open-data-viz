<script>
  import { onMount, untrack } from "svelte";
  /** @typedef {import("./constants/types.js").GeoCollection} GeoCollection */
  /** @typedef {import("./constants/types.js").PrefectureMonth} PrefectureMonth */
  import {
    createOkinawaInsetGeojson,
    observationLabel,
    prefectureKey,
  } from "./utils/relayMap.js";

  /** @typedef {import("./utils/relayMap.js").LatLng} LatLng */
  /** @typedef {import("leaflet").Map} LeafletMap */
  /** @typedef {import("leaflet").GeoJSON} LeafletGeoJSON */
  /** @typedef {import("leaflet").LayerGroup} LeafletLayerGroup */
  /** @typedef {import("leaflet").Polyline} LeafletPolyline */
  /** @typedef {import("leaflet").CircleMarker} LeafletCircleMarker */
  /** @typedef {import("leaflet").Marker} LeafletSquareMarker */

  const OKINAWA_INSET = {
    center: /** @type {LatLng} */ ([41, 131]),
    separator: /** @type {LatLng[]} */ ([
      [38, 126],
      [38, 132],
      [42, 137],
    ]),
  };
  /** @type {import("leaflet").TooltipOptions} */
  const TOOLTIP_OPTIONS = { direction: "top", sticky: true };
  const MARKER_ANIMATION_MS = 125;
  const MARKER_FILL_OPACITY = 0.85;
  const MARKER_RADIUS_DIVISOR = 15;
  // A square with half-side r * sqrt(PI)/2 has the same area as a circle of radius r.
  const SQUARE_HALF_SIDE_FACTOR = Math.sqrt(Math.PI) / 2;
  /** @type {[number, number]} */
  const MAP_FIT_PADDING = [12, 12];

  /** @type {{geojson: GeoCollection, selectedProduct: string, selectedMonth: string, observationsByPrefecture: Record<string, PrefectureMonth>, sourceUnit: string | null, scale: number, markerShape: string, onSelect: (name: string) => void}} */
  let {
    geojson,
    selectedProduct,
    selectedMonth,
    observationsByPrefecture,
    sourceUnit,
    scale,
    markerShape,
    onSelect,
  } = $props();

  /** @type {number | undefined} */
  let previousMonthNumber;

  /** @type {HTMLDivElement} */
  let host;
  let mapReady = $state(false);
  /** @type {typeof import("leaflet") | undefined} */
  let leaflet;
  /** @type {LeafletMap | undefined} */
  let map;
  /** @type {LeafletGeoJSON | undefined} */
  let boundary;
  /** @type {LeafletLayerGroup | undefined} */
  let markerLayer;
  /** @type {LeafletPolyline | undefined} */
  let insetSeparator;
  /** @type {{boundary: string, boundaryFill: string, marker: string, markerFill: string} | undefined} */
  let mapColors;
  /** @type {Map<string, LeafletCircleMarker | LeafletSquareMarker>} */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- Leaflet bookkeeping must not retrigger the drawing effect.
  const markers = new Map();
  /** @type {Map<string, number>} */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- Animation frames are external imperative state.
  const markerAnimations = new Map();
  /** @type {Map<string, {radius: number, opacity: number}>} */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- Animation targets are external imperative state.
  const markerTargets = new Map();
  // Geographic data is recomputed only when the GeoJSON object changes.
  /** @type {Map<string, LatLng>} */
  // eslint-disable-next-line svelte/prefer-svelte-reactivity -- Cached Leaflet coordinates are external imperative state.
  const centroidLocations = new Map();
  let geometryVersion = $state(0);
  let productFitPending = $state(false);
  /** @type {import("leaflet").LatLngBounds | undefined} */
  let mapBounds;
  /** @type {string | undefined} */
  let previousProduct;

  function readMapColors() {
    const styles = getComputedStyle(host);
    return {
      boundary: styles.getPropertyValue("--map-boundary-color").trim(),
      boundaryFill: styles.getPropertyValue("--map-boundary-fill-color").trim(),
      marker: styles.getPropertyValue("--map-marker-color").trim(),
      markerFill: styles.getPropertyValue("--map-marker-fill-color").trim(),
    };
  }

  /** @param {string} name */
  function cancelMarkerAnimation(name) {
    const frame = markerAnimations.get(name);
    if (frame !== undefined) cancelAnimationFrame(frame);
    markerAnimations.delete(name);
  }

  /** @param {string} name */
  function removeMarker(name) {
    cancelMarkerAnimation(name);
    markerTargets.delete(name);
    const marker = markers.get(name);
    if (marker) markerLayer?.removeLayer(marker);
    markers.delete(name);
  }

  function clearMarkers() {
    for (const name of markers.keys()) removeMarker(name);
  }

  /**
   * Shared animation clock for circular and square markers. Each shape
   * supplies its own renderer, so interpolation and cancellation stay in sync.
   * @param {string} name
   * @param {number} startRadius
   * @param {number} startOpacity
   * @param {number} targetRadius
   * @param {number} targetOpacity
   * @param {(radius: number, opacity: number) => void} render
   */
  function tweenMarker(
    name,
    startRadius,
    startOpacity,
    targetRadius,
    targetOpacity,
    render,
  ) {
    const startTime = performance.now();
    /** @param {number} now */
    const step = (now) => {
      const progress = Math.min((now - startTime) / MARKER_ANIMATION_MS, 1);
      const eased =
        progress < 0.5 ? 4 * progress ** 3 : 1 - (-2 * progress + 2) ** 3 / 2;
      render(
        startRadius + (targetRadius - startRadius) * eased,
        startOpacity + (targetOpacity - startOpacity) * eased,
      );
      if (progress < 1) {
        markerAnimations.set(name, requestAnimationFrame(step));
      } else {
        markerAnimations.delete(name);
      }
    };
    markerAnimations.set(name, requestAnimationFrame(step));
  }

  /** @param {string} name @param {LeafletCircleMarker} marker @param {number} targetRadius @param {number} targetOpacity @param {boolean} animate */
  function animateMarker(name, marker, targetRadius, targetOpacity, animate) {
    const previousTarget = markerTargets.get(name);
    const hasActiveAnimation = markerAnimations.has(name);
    if (
      animate &&
      previousTarget?.radius === targetRadius &&
      previousTarget.opacity === targetOpacity &&
      (hasActiveAnimation || Math.abs(marker.getRadius() - targetRadius) < 0.01)
    ) {
      return;
    }
    markerTargets.set(name, { radius: targetRadius, opacity: targetOpacity });

    if (!animate || hasActiveAnimation) {
      // Prefer the latest value when updates arrive faster than the animation.
      cancelMarkerAnimation(name);
      marker.setRadius(targetRadius);
      marker.setStyle({ fillOpacity: targetOpacity });
      return;
    }

    tweenMarker(
      name,
      marker.getRadius(),
      Number(marker.options.fillOpacity ?? MARKER_FILL_OPACITY),
      targetRadius,
      targetOpacity,
      (radius, opacity) => {
        marker.setRadius(radius);
        marker.setStyle({ fillOpacity: opacity });
      },
    );
  }

  /** @param {string} name @param {LeafletSquareMarker} marker @param {number} targetHalfSide @param {number} targetOpacity @param {boolean} animate */
  function animateSquare(name, marker, targetHalfSide, targetOpacity, animate) {
    // The div icon remains the same size in CSS pixels as Leaflet zooms.
    // Its graphic is centered on the Marker anchor, like a CircleMarker.
    const graphic = marker.getElement()?.querySelector(".relay-square-graphic");
    const fill = graphic?.querySelector(".relay-square-fill");
    if (!(graphic instanceof HTMLElement) || !(fill instanceof HTMLElement))
      return;

    const startHalfSide = (Number.parseFloat(graphic.style.width) || 0) / 2;
    const startOpacity = Number(fill.style.opacity || 0);
    const previousTarget = markerTargets.get(name);
    const hasActiveAnimation = markerAnimations.has(name);
    if (
      animate &&
      previousTarget?.radius === targetHalfSide &&
      previousTarget.opacity === targetOpacity &&
      (hasActiveAnimation || Math.abs(startHalfSide - targetHalfSide) < 0.01)
    )
      return;

    markerTargets.set(name, { radius: targetHalfSide, opacity: targetOpacity });
    cancelMarkerAnimation(name);

    /** @param {number} halfSide @param {number} opacity */
    const render = (halfSide, opacity) => {
      const size = halfSide * 2;
      graphic.style.width = `${size}px`;
      graphic.style.height = `${size}px`;
      fill.style.opacity = String(opacity);
      // Leaflet uses these values to pan the full marker extent into view on focus.
      const icon = marker.options.icon;
      if (icon) {
        icon.options.iconSize = [size, size];
        icon.options.iconAnchor = [halfSide, halfSide];
      }
    };

    // A newer update supersedes an in-flight animation, matching circles.
    if (
      !animate ||
      hasActiveAnimation ||
      Math.abs(startHalfSide - targetHalfSide) < 0.01
    ) {
      render(targetHalfSide, targetOpacity);
      return;
    }

    tweenMarker(
      name,
      startHalfSide,
      startOpacity,
      targetHalfSide,
      targetOpacity,
      render,
    );
  }

  /**
   * @param {typeof import("leaflet")} L
   * @param {LeafletLayerGroup} layerGroup
   * @param {string} name
   * @param {LatLng} location
   * @param {string} label
   * @param {string} shape
   * @param {{marker: string, markerFill: string}} colors
   */
  function createMarker(L, layerGroup, name, location, label, shape, colors) {
    const marker =
      shape === "square"
        ? L.marker(location, {
            icon: L.divIcon({
              className: "relay-square-icon",
              iconSize: [0, 0],
              iconAnchor: [0, 0],
              html: '<span class="relay-square-graphic"><span class="relay-square-fill"></span></span>',
            }),
          })
        : L.circleMarker(location, {
            color: colors.marker,
            weight: 1,
            fillColor: colors.markerFill,
            radius: 0,
            fillOpacity: 0,
          });
    marker
      .bindTooltip(label, TOOLTIP_OPTIONS)
      .on("click", () => onSelect(name))
      .addTo(layerGroup);
    if (shape === "square") {
      // Keep the border opaque; animate only the fill, as with CircleMarker.
      const icon = marker.getElement();
      if (icon instanceof HTMLElement) {
        icon.setAttribute("aria-label", label);
        icon.style.setProperty("--relay-square-border", colors.marker);
        icon.style.setProperty("--relay-square-background", colors.markerFill);
      }
    }
    return marker;
  }

  onMount(() => {
    let cancelled = false;
    const resizeObserver = new ResizeObserver(() => map?.invalidateSize());
    resizeObserver.observe(host);
    void (async () => {
      const L = await import("leaflet");
      await import("leaflet/dist/leaflet.css");
      if (cancelled || !host.isConnected) return;
      leaflet = L;
      map = L.map(host, {
        zoomControl: false,
        attributionControl: true,
        zoomSnap: 0.5,
        scrollWheelZoom: true,
        wheelPxPerZoomLevel: 240,
      });
      L.control.zoom({ position: "bottomright" }).addTo(map);
      map.attributionControl.setPrefix("Leaflet");
      map.attributionControl.addAttribution(
        '<a href="https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2026.html" target="_blank" rel="noreferrer">国土交通省「国土数値情報 行政区域データ（N03）」2026年版</a>',
      );
      map.setView([37, 137], 5);
      mapColors = readMapColors();
      mapReady = true;
    })();
    return () => {
      cancelled = true;
      resizeObserver.disconnect();
      mapReady = false;
      mapColors = undefined;
      clearMarkers();
      map?.remove();
      map = undefined;
    };
  });

  // Static geometry: rebuild boundaries and centroid coordinates only when
  // the map is initialized or a different GeoJSON object is supplied.
  $effect(() => {
    if (!mapReady || !leaflet || !map || !geojson || !mapColors) return;
    untrack(() => {
      geometryVersion = 0;
    });

    const colors = mapColors;
    const {
      geojson: displayGeojson,
      locations,
      offset,
    } = createOkinawaInsetGeojson(geojson, OKINAWA_INSET.center);
    boundary?.remove();
    boundary = leaflet
      .geoJSON(/** @type {any} */ (displayGeojson), {
        style: () => ({
          color: colors.boundary,
          weight: 1,
          fillColor: colors.boundaryFill,
          fillOpacity: 1,
        }),
        onEachFeature: (feature, layer) => {
          const name = prefectureKey(
            String(feature.properties?.N03_001 ?? "").trim(),
          );
          // The observation effect supplies the actual label.
          layer.bindTooltip(`${name}: 値なし`, TOOLTIP_OPTIONS);
          layer.on("click", () => onSelect(name));
        },
      })
      .addTo(map);

    insetSeparator?.remove();
    insetSeparator = undefined;
    if (offset) {
      insetSeparator = leaflet
        .polyline(OKINAWA_INSET.separator, {
          color: colors.boundary,
          weight: 2,
          opacity: 0.8,
          interactive: false,
        })
        .addTo(map);
    }

    markerLayer ??= leaflet.layerGroup().addTo(map);
    centroidLocations.clear();
    for (const [name, location] of locations) {
      centroidLocations.set(
        name,
        name === "沖縄" && offset
          ? [location[0] + offset.latitude, location[1] + offset.longitude]
          : location,
      );
    }

    // Existing markers must stay above the replacement boundary layer.
    // Marker changes must not retrigger geometry setup and reset the viewport.
    untrack(() => {
      for (const marker of markers.values()) {
        // DOM icon markers are in Leaflet's marker pane and have no bringToFront().
        if ("bringToFront" in marker) marker.bringToFront();
      }
    });

    mapBounds = boundary.getBounds();
    if (insetSeparator) mapBounds.extend(insetSeparator.getBounds());
    const finishGeometrySetup = () => {
      untrack(() => {
        geometryVersion += 1;
      });
    };
    if (mapBounds.isValid()) {
      // Add circles only after Leaflet finishes fitting the map to the bounds.
      map.once("moveend", finishGeometrySetup);
      map.fitBounds(mapBounds, { padding: MAP_FIT_PADDING });
    } else {
      finishGeometrySetup();
    }
  });

  // Keep the Leaflet map and boundary layers mounted when the product changes.
  // Refit the existing map, then let the observation effect draw the new circles.
  $effect(() => {
    if (!mapReady || !map || !geometryVersion || !mapBounds) return;
    if (previousProduct === undefined) {
      previousProduct = selectedProduct;
      return;
    }
    if (previousProduct === selectedProduct) return;
    previousProduct = selectedProduct;

    clearMarkers();

    productFitPending = true;
    map.once("moveend", () => {
      productFitPending = false;
    });
    map.fitBounds(mapBounds, { padding: MAP_FIT_PADDING });
  });

  // Dynamic data: refresh labels, positions, and marker animations without
  // touching boundary polygons or recalculating geographic centroids.
  $effect(() => {
    if (
      !mapReady ||
      !leaflet ||
      !map ||
      !markerLayer ||
      !geometryVersion ||
      productFitPending ||
      !mapColors
    )
      return;
    const colors = mapColors;
    const currentObservations = observationsByPrefecture;
    const currentScale = scale;
    const currentShape = markerShape;
    const [year, month] = selectedMonth.split("-").map(Number);
    const monthNumber = year * 12 + month;
    const animate =
      previousMonthNumber === undefined ||
      Math.abs(monthNumber - previousMonthNumber) <= 1;
    previousMonthNumber = monthNumber;

    boundary?.eachLayer((layer) => {
      const feature =
        /** @type {import("leaflet").Layer & {feature?: {properties?: {N03_001?: string}}} } */ (
          layer
        ).feature;
      const name = prefectureKey(
        String(feature?.properties?.N03_001 ?? "").trim(),
      );
      const observation = currentObservations[name];
      const label = `${name}: ${observationLabel(observation, sourceUnit)}`;
      layer.getTooltip()?.setContent(label);
    });

    /** @type {Set<string>} */
    // eslint-disable-next-line svelte/prefer-svelte-reactivity -- This set is local to one drawing pass.
    const activeNames = new Set();
    for (const [name, location] of centroidLocations) {
      const observation = currentObservations[name];
      const value = Number(observation?.reported_value_sum ?? 0);
      if (!observation?.reported_cells || value <= 0) continue;
      activeNames.add(name);
      const targetRadius =
        (Math.sqrt(value) * currentScale) / MARKER_RADIUS_DIVISOR;
      const label = `${name}: ${observationLabel(observation, sourceUnit)}`;
      let marker = markers.get(name);
      // Changing shapes requires a new Leaflet layer, not a new map.
      if (marker && "getRadius" in marker !== (currentShape === "circle")) {
        removeMarker(name);
        marker = undefined;
      }
      if (!marker) {
        marker = createMarker(
          leaflet,
          markerLayer,
          name,
          location,
          label,
          currentShape,
          colors,
        );
        markers.set(name, marker);
      } else {
        marker.setTooltipContent(label);
      }
      if (!("getRadius" in marker)) {
        marker.getElement()?.setAttribute("aria-label", label);
      }

      marker.setLatLng(location);
      if ("getRadius" in marker) {
        animateMarker(name, marker, targetRadius, MARKER_FILL_OPACITY, animate);
      } else {
        animateSquare(
          name,
          marker,
          targetRadius * SQUARE_HALF_SIDE_FACTOR,
          MARKER_FILL_OPACITY,
          animate,
        );
      }
    }

    for (const name of markers.keys()) {
      if (!activeNames.has(name)) removeMarker(name);
    }
  });
</script>

<div
  class="map"
  bind:this={host}
  aria-label="都道府県別の野菜卸売数量地図"
></div>

<style>
  .map {
    box-sizing: border-box;
    height: var(--relay-map-height, min(68vh, 680px));
    min-height: var(--relay-map-min-height, 400px);
    border-radius: var(--relay-map-radius, var(--radius-md));
    background: var(--map-surface-color);
  }
  /* DivIcon content is pixel-sized, so squares never grow when the map zooms. */
  :global(.relay-square-icon) {
    overflow: visible;
    border: none;
    background: transparent;
  }
  :global(.relay-square-graphic) {
    position: absolute;
    top: 0;
    left: 0;
    box-sizing: border-box;
    transform: translate(-50%, -50%);
    border: 1px solid var(--relay-square-border);
  }
  :global(.relay-square-icon:focus-visible .relay-square-graphic) {
    outline: 2px solid var(--focus-color);
    outline-offset: 2px;
  }
  :global(.relay-square-fill) {
    position: absolute;
    inset: 0;
    background: var(--relay-square-background);
    opacity: 0;
    pointer-events: none;
  }
  /* Map height adjustment inside the map; page columns use the shared breakpoint. */
  @media (max-width: 600px) {
    .map {
      height: var(--relay-map-height, 55vh);
      min-height: var(--relay-map-min-height, 360px);
    }
  }
</style>
