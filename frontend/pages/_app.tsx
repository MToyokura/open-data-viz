import type { AppProps } from "next/app";
import Link from "next/link";
import { ByIcon, CcIcon } from "../page_modules/components/SvgIcons";
import "../styles/globals.css";

export const appName = "Open Data Viz";
export const staticAssetsBaseUrl = getStaticAssetsBaseUrl(process.env.NODE_ENV);

function getStaticAssetsBaseUrl(nodeEnv: NodeJS.Process["env"]["NODE_ENV"]) {
  if (nodeEnv === "production") {
    return "https://storage.googleapis.com/open-data-viz/public/assets";
  } else {
    return "/assets";
  }
}

function MyApp({ Component, pageProps }: AppProps) {
  return (
    <>
      <div style={{ minHeight: "85vh" }}>
        <div
          style={{
            backgroundColor: "rgba(0, 0, 0, 0.1)",
            padding: "1rem",
          }}
        >
          <div style={{ display: "flex" }}>
            <Link href="/" style={{ color: "inherit", textDecoration: "none" }}>
              <div
                style={{
                  display: "flex",
                  alignItems: "center",
                  gap: "0.5rem",
                }}
              >
                <img
                  style={{ height: "2rem" }}
                  src="../site_logo.png"
                  alt="Website Logo"
                />
                <div style={{ fontSize: "1.5rem" }}>{appName}</div>
              </div>
            </Link>
          </div>
        </div>
        <Component {...pageProps} />
      </div>
      <div
        className="footer"
        style={{
          display: "flex",
          justifyContent: "end",
          margin: "3rem 0.5rem 0.5rem 0.5rem",
        }}
      >
        <a
          href="https://creativecommons.org/licenses/by/4.0/deed.ja"
          target="_blank"
          rel="noreferrer"
        >
          <div
            style={{
              display: "flex",
              height: "1.5rem",
              gap: "0.3rem",
            }}
          >
            <CcIcon fillColor="grey" />
            <ByIcon fillColor="grey" />
          </div>
        </a>
      </div>
    </>
  );
}

export default MyApp;
