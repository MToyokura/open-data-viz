<script lang="ts">
  import { onMount, untrack } from "svelte";

  import type { GeoCollection, PrefectureMonth } from "./constants/types";
  import {
    calculateCentroids,
    observationLabel,
    prefectureKey,
    translateCoordinates,
    type LatLng,
  } from "./utils/relayMap";

  const OKINAWA_INSET = {
    center: [41, 131] as LatLng,
    separator: [
      [38, 126],
      [38, 132],
      [42, 137],
    ] as [LatLng, LatLng, LatLng],
  } as const;

  let {
    geojson,
    observations,
    sourceUnit,
    scale,
    onSelect,
  }: {
    geojson: GeoCollection;
    observations: Record<string, PrefectureMonth>;
    sourceUnit: string | null;
    scale: number;
    onSelect: (name: string) => void;
  } = $props();

  let host: HTMLDivElement;
  let mapReady = $state(false);
  let leaflet: typeof import("leaflet") | undefined;
  let map: import("leaflet").Map | undefined;
  let boundary: import("leaflet").GeoJSON | undefined;
  let circles: import("leaflet").LayerGroup | undefined;
  let insetSeparator: import("leaflet").Polyline | undefined;
  let mapColors:
    | {
        boundary: string;
        boundaryFill: string;
        marker: string;
        markerFill: string;
      }
    | undefined;
  const markers = new Map<string, import("leaflet").CircleMarker>();
  const markerAnimations = new Map<string, number>();
  const markerTargets = new Map<string, { radius: number; opacity: number }>();
  // Geographic data is recomputed only when the GeoJSON object changes.
  const centroidLocations = new Map<string, [number, number]>();
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

  function animateMarker(
    name: string,
    marker: import("leaflet").CircleMarker,
    targetRadius: number,
    targetOpacity: number,
  ) {
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
    const duration = 300;

    const step = (now: number) => {
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
    void (async () => {
      const L = await import("leaflet");
      await import("leaflet/dist/leaflet.css");
      if (cancelled || !host.isConnected) return;
      leaflet = L;
      map = L.map(host, {
        zoomControl: true,
        attributionControl: true,
        zoomSnap: 0.5,
        scrollWheelZoom: true,
        wheelPxPerZoomLevel: 240,
      });
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
    const originalLocations = calculateCentroids(geojson.features);
    const okinawa = originalLocations.get("沖縄");
    const offset = okinawa
      ? {
          latitude: OKINAWA_INSET.center[0] - okinawa[0],
          longitude: OKINAWA_INSET.center[1] - okinawa[1],
        }
      : null;
    const displayGeojson: GeoCollection = offset
      ? {
          ...geojson,
          features: geojson.features.map((feature) => {
            const name = prefectureKey(
              String(feature.properties?.N03_001 ?? "").trim(),
            );
            if (name !== "沖縄") return feature;

            const geometry = feature.geometry as {
              type: string;
              coordinates: unknown;
            };
            if (geometry.type !== "Polygon" && geometry.type !== "MultiPolygon")
              return feature;

            return {
              ...feature,
              geometry: {
                ...geometry,
                coordinates: translateCoordinates(
                  geometry.coordinates,
                  offset.longitude,
                  offset.latitude,
                ),
              },
            };
          }),
        }
      : geojson;
    boundary?.remove();
    boundary = leaflet
      .geoJSON(displayGeojson as never, {
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
          layer.bindTooltip(`${name}: 値なし`);
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
    for (const [name, location] of originalLocations) {
      centroidLocations.set(
        name,
        name === "沖縄" && offset
          ? [location[0] + offset.latitude, location[1] + offset.longitude]
          : location,
      );
    }

    // Existing markers must stay above the replacement boundary layer.
    for (const marker of markers.values()) marker.bringToFront();

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
    const currentObservations = observations;
    const currentScale = scale;

    boundary?.eachLayer((layer) => {
      const feature = (
        layer as import("leaflet").Layer & {
          feature?: { properties?: { N03_001?: string } };
        }
      ).feature;
      const name = prefectureKey(
        String(feature?.properties?.N03_001 ?? "").trim(),
      );
      const observation = currentObservations[name];
      const label = `${name}: ${observationLabel(observation, sourceUnit)}`;
      layer.getTooltip()?.setContent(label);
    });

    const activeNames = new Set<string>();
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
          .bindTooltip(`${name}: ${observationLabel(observation, sourceUnit)}`)
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
    height: min(68vh, 680px);
    min-height: 400px;
    border: var(--border-default);
    border-radius: var(--radius-md);
    background: var(--map-surface-color);
  }
  @media (max-width: 600px) {
    .map {
      height: 55vh;
      min-height: 360px;
    }
  }
</style>
