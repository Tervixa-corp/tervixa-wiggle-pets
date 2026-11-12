import { Config } from "@remotion/cli/config";

// Deterministic, high-quality defaults for a 1080p60 master.
Config.setVideoImageFormat("jpeg");
Config.setJpegQuality(96);
Config.setConcurrency(8);
Config.setChromiumOpenGlRenderer("swangle");
Config.setDelayRenderTimeoutInMilliseconds(120000);
