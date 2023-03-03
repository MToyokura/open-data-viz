import type { AppProps } from "next/app";
import Link from "next/link";
import { useRouter } from "next/router";
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
  const router = useRouter();
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
                  // https://stackoverflow.com/questions/64921521/how-to-change-base-path-for-assets-images-etc/65113234#65113234
                  // https://stackoverflow.com/questions/68194255/how-to-obtain-a-path-without-using-link-when-basepath-is-set-in-next-js/68201934#68201934
                  src={`${router.basePath}/site_logo.png`}
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
