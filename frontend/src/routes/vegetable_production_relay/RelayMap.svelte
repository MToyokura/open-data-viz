<script>
  import { onMount, untrack } from "svelte";
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
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

  /** @type {{geojson: GeoCollection, observationsByPrefecture: Record<string, PrefectureMonth>, sourceUnit: string | null, scale: number, onSelect: (name: string) => void}} */
  let { geojson, observationsByPrefecture, sourceUnit, scale, onSelect } =
    $props();

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
  let circles;
  /** @type {LeafletPolyline | undefined} */
  let insetSeparator;
  /** @type {{boundary: string, boundaryFill: string, marker: string, markerFill: string} | undefined} */
  let mapColors;
  /** @type {SvelteMap<string, LeafletCircleMarker>} */
  const markers = new SvelteMap();
  /** @type {SvelteMap<string, number>} */
  const markerAnimations = new SvelteMap();
  /** @type {SvelteMap<string, {radius: number, opacity: number}>} */
  const markerTargets = new SvelteMap();
  // Geographic data is recomputed only when the GeoJSON object changes.
  /** @type {SvelteMap<string, LatLng>} */
  const centroidLocations = new SvelteMap();
  let geometryVersion = $state(0);

  function readMapColors() {
    const styles = getComputedStyle(host);
    return {
      boundary: styles.getPropertyValue("--map-boundary-color").trim(),
      boundaryFill: styles.getPropertyValue("--map-boundary-fill-color").trim(),
      marker: styles.getPropertyValue("--map-marker-color").trim(),
      markerFill: styles.getPropertyValue("--map-marker-fill-color").trim(),
    };
  }

  /** @param {string} name @param {LeafletCircleMarker} marker @param {number} targetRadius @param {number} targetOpacity */
  function animateMarker(name, marker, targetRadius, targetOpacity) {
    const previousTarget = markerTargets.get(name);
    const hasActiveAnimation = markerAnimations.has(name);
    if (
      previousTarget?.radius === targetRadius &&
      previousTarget.opacity === targetOpacity &&
      (hasActiveAnimation || Math.abs(marker.getRadius() - targetRadius) < 0.01)
    ) {
      return;
    }
    markerTargets.set(name, { radius: targetRadius, opacity: targetOpacity });

    const previousFrame = markerAnimations.get(name);
    if (previousFrame !== undefined) cancelAnimationFrame(previousFrame);

    const startRadius = marker.getRadius();
    const startOpacity = Number(marker.options.fillOpacity ?? 0.85);
    const startTime = performance.now();
    const duration = 200;

    /** @param {number} now */
    const step = (now) => {
      const progress = Math.min((now - startTime) / duration, 1);
      const eased =
        progress < 0.5 ? 4 * progress ** 3 : 1 - (-2 * progress + 2) ** 3 / 2;
      marker.setRadius(startRadius + (targetRadius - startRadius) * eased);
      marker.setStyle({
        fillOpacity: startOpacity + (targetOpacity - startOpacity) * eased,
      });

      if (progress < 1) {
        markerAnimations.set(name, requestAnimationFrame(step));
      } else {
        markerAnimations.delete(name);
      }
    };

    markerAnimations.set(name, requestAnimationFrame(step));
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
      for (const frame of markerAnimations.values()) {
        cancelAnimationFrame(frame);
      }
      markerAnimations.clear();
      markers.clear();
      markerTargets.clear();
      map?.remove();
      map = undefined;
    };
  });

  // Static geometry: rebuild boundaries and centroid coordinates only when
  // the map is initialized or a different GeoJSON object is supplied.
  $effect(() => {
    if (!mapReady || !leaflet || !map || !geojson || !mapColors) return;

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

    circles ??= leaflet.layerGroup().addTo(map);
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
      for (const marker of markers.values()) marker.bringToFront();
    });

    const bounds = boundary.getBounds();
    if (insetSeparator) bounds.extend(insetSeparator.getBounds());
    if (bounds.isValid()) map.fitBounds(bounds, { padding: [12, 12] });

    // Signal that the marker effect should consume the new locations.
    untrack(() => {
      geometryVersion += 1;
    });
  });

  // Dynamic data: refresh labels, positions, and marker animations without
  // touching boundary polygons or recalculating geographic centroids.
  $effect(() => {
    if (
      !mapReady ||
      !leaflet ||
      !map ||
      !circles ||
      !geometryVersion ||
      !mapColors
    )
      return;
    const colors = mapColors;
    const currentObservations = observationsByPrefecture;
    const currentScale = scale;

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

    /** @type {SvelteSet<string>} */
    const activeNames = new SvelteSet();
    for (const [name, location] of centroidLocations) {
      const observation = currentObservations[name];
      const value = Number(observation?.reported_value_sum ?? 0);
      if (!observation?.reported_cells || value <= 0) continue;
      activeNames.add(name);
      const targetRadius = (Math.sqrt(value) * currentScale) / 15;
      let marker = markers.get(name);

      if (marker) {
        marker.setLatLng(location);
        marker.setTooltipContent(
          `${name}: ${observationLabel(observation, sourceUnit)}`,
        );
      } else {
        marker = leaflet.circleMarker(location, {
          radius: 0,
          color: colors.marker,
          weight: 1,
          fillColor: colors.markerFill,
          fillOpacity: 0,
        });
        marker
          .bindTooltip(
            `${name}: ${observationLabel(observation, sourceUnit)}`,
            TOOLTIP_OPTIONS,
          )
          .on("click", () => onSelect(name))
          .addTo(circles);
        markers.set(name, marker);
      }

      animateMarker(name, marker, targetRadius, 0.85);
    }

    for (const [name, marker] of markers) {
      if (activeNames.has(name)) continue;
      const frame = markerAnimations.get(name);
      if (frame !== undefined) cancelAnimationFrame(frame);
      markerAnimations.delete(name);
      markerTargets.delete(name);
      marker.remove();
      markers.delete(name);
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
  @media (max-width: 600px) {
    .map {
      height: var(--relay-map-height, 55vh);
      min-height: var(--relay-map-min-height, 360px);
    }
  }
</style>
