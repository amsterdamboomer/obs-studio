#include "frontend/build/frontend-utility-profile-activate.pch.h"

void profile_activate_config(const struct profile_activate_ops *ops, bool reset)
{
	ops->init_defaults(ops->param);

	/* Defaults live in the loaded configuration only, and a renamed or
	 * duplicated profile is a new copy of basic.ini: set the encoder
	 * defaults on every activation, or keys the profile never saved read
	 * as NULL. */
	ops->init_encoder_defaults(ops->param);

	if (reset) {
		ops->check_encoders(ops->param);
		ops->reset_data(ops->param);
	}
}

void profile_set_encoder_defaults(config_t *config, const char *simple_encoder, const char *aac_encoder)
{
	config_set_default_string(config, "SimpleOutput", "StreamEncoder", simple_encoder);
	config_set_default_string(config, "SimpleOutput", "RecEncoder", simple_encoder);

	config_set_default_string(config, "AdvOut", "AudioEncoder", aac_encoder);
	config_set_default_string(config, "AdvOut", "RecAudioEncoder", aac_encoder);
}

static bool profile_encoder_id_in(const char *id, const char *const *ids)
{
	for (; *ids; ids++) {
		const char *a = id;
		const char *b = *ids;
		while (*a && *a == *b) {
			a++;
			b++;
		}
		if (*a == '\0' && *b == '\0') {
			return true;
		}
	}
	return false;
}

/* The encoders plugins/obs-qsv11 and plugins/obs-ffmpeg's VA-API encoder
 * register, by id. */
static const char *const profile_qsv_encoder_ids[] = {
	"obs_qsv11",         "obs_qsv11_v2",       "obs_qsv11_soft",
	"obs_qsv11_soft_v2", "obs_qsv11_hevc",     "obs_qsv11_hevc_soft",
	"obs_qsv11_av1",     "obs_qsv11_av1_soft", NULL,
};
static const char *const profile_vaapi_encoder_ids[] = {
	"ffmpeg_vaapi",
	"ffmpeg_vaapi_tex",
	"hevc_ffmpeg_vaapi",
	"hevc_ffmpeg_vaapi_tex",
	"av1_ffmpeg_vaapi",
	"av1_ffmpeg_vaapi_tex",
	NULL,
};

enum profile_missing_encoder
profile_missing_encoder_reason(const char *encoder_id, bool (*module_loaded)(const char *name), const char **module)
{
	const char *name = NULL;
	enum profile_missing_encoder no_device = PROFILE_MISSING_ENCODER_UNKNOWN;

	if (encoder_id && profile_encoder_id_in(encoder_id, profile_qsv_encoder_ids)) {
		name = "obs-qsv11";
		no_device = PROFILE_MISSING_ENCODER_NO_QSV_DEVICE;
	} else if (encoder_id && profile_encoder_id_in(encoder_id, profile_vaapi_encoder_ids)) {
		name = "obs-ffmpeg";
		no_device = PROFILE_MISSING_ENCODER_NO_VAAPI_DEVICE;
	}

	if (module) {
		*module = name;
	}
	if (!name) {
		return PROFILE_MISSING_ENCODER_UNKNOWN;
	}
	return module_loaded(name) ? no_device : PROFILE_MISSING_ENCODER_MODULE_NOT_LOADED;
}

static bool profile_vaapi_read_file(void *param, const char *path, char *buf, size_t size)
{
	(void)param;
	FILE *f = fopen(path, "rb");
	size_t n;
	if (!f) {
		return false;
	}
	n = fread(buf, 1, size - 1, f);
	buf[n] = '\0';
	fclose(f);
	return true;
}

static bool profile_vaapi_exists(void *param, const char *path)
{
	(void)param;
	FILE *f = fopen(path, "rb");
	if (!f) {
		return false;
	}
	fclose(f);
	return true;
}

static bool profile_vaapi_can_open_rw(void *param, const char *path)
{
	(void)param;
	FILE *f = fopen(path, "r+b");
	if (!f) {
		return false;
	}
	fclose(f);
	return true;
}

const struct profile_vaapi_host profile_vaapi_system_host = {
	NULL,
	profile_vaapi_read_file,
	profile_vaapi_exists,
	profile_vaapi_can_open_rw,
};

#define PROFILE_PCI_VENDOR_INTEL 0x8086u
#define PROFILE_PCI_VENDOR_AMD 0x1002u
#define PROFILE_PCI_VENDOR_NVIDIA 0x10deu

/* libva's render device range, and where distributions install VA-API
 * drivers (hostlib's search dirs, build-aux/appimage/hostlib/hostlib.c). */
#define PROFILE_RENDER_NODE_FIRST 128
#define PROFILE_RENDER_NODE_COUNT 8

static const char *const profile_vaapi_driver_dirs[] = {
	"/run/opengl-driver/lib/dri",
	"/usr/lib/x86_64-linux-gnu/dri",
	"/usr/lib/aarch64-linux-gnu/dri",
	"/usr/lib64/dri",
	"/usr/lib/dri",
	NULL,
};

enum profile_distro {
	PROFILE_DISTRO_UNKNOWN,
	PROFILE_DISTRO_DEBIAN,
	PROFILE_DISTRO_FEDORA,
	PROFILE_DISTRO_ARCH,
};

struct profile_vaapi_vendor {
	unsigned int vendor;
	const char *const drivers[3];
	/* By enum profile_distro; [PROFILE_DISTRO_UNKNOWN] is the name shown
	 * when the distribution is not recognized. */
	const char *packages[4];
};

static const struct profile_vaapi_vendor profile_vaapi_vendors[] = {
	{PROFILE_PCI_VENDOR_INTEL,
	 {"iHD", "i965", NULL},
	 {"intel-media-driver", "intel-media-va-driver-non-free", "intel-media-driver", "intel-media-driver"}},
	{PROFILE_PCI_VENDOR_AMD,
	 {"radeonsi", NULL, NULL},
	 {"mesa-va-drivers", "mesa-va-drivers", "mesa-va-drivers-freeworld", "mesa"}},
};

static const char *const profile_install_prefix[] = {NULL, "sudo apt install ", "sudo dnf install ", "sudo pacman -S "};

/* Whether the space-separated, optionally quoted value of key (ID or
 * ID_LIKE) in os-release text names id. */
static bool profile_os_release_has(const char *text, const char *key, const char *id)
{
	size_t key_len = strlen(key);
	size_t id_len = strlen(id);
	const char *line = text;

	while (line && *line) {
		if (strncmp(line, key, key_len) == 0 && line[key_len] == '=') {
			const char *p = line + key_len + 1;
			while (*p && *p != '\n') {
				while (*p == ' ' || *p == '"' || *p == '\'') {
					p++;
				}
				const char *word = p;
				while (*p && *p != '\n' && *p != ' ' && *p != '"' && *p != '\'') {
					p++;
				}
				if ((size_t)(p - word) == id_len && strncmp(word, id, id_len) == 0) {
					return true;
				}
				if (*p == '"' || *p == '\'') {
					p++;
				}
			}
		}
		line = strchr(line, '\n');
		if (line) {
			line++;
		}
	}
	return false;
}

static enum profile_distro profile_detect_distro(const struct profile_vaapi_host *host)
{
	static const struct {
		const char *id;
		enum profile_distro distro;
	} ids[] = {
		{"debian", PROFILE_DISTRO_DEBIAN}, {"ubuntu", PROFILE_DISTRO_DEBIAN}, {"fedora", PROFILE_DISTRO_FEDORA},
		{"rhel", PROFILE_DISTRO_FEDORA},   {"arch", PROFILE_DISTRO_ARCH},
	};
	char text[2048];

	if (!host->read_file(host->param, "/etc/os-release", text, sizeof(text))) {
		return PROFILE_DISTRO_UNKNOWN;
	}
	for (size_t k = 0; k < 2; k++) {
		const char *key = k == 0 ? "ID" : "ID_LIKE";
		for (size_t i = 0; i < sizeof(ids) / sizeof(ids[0]); i++) {
			if (profile_os_release_has(text, key, ids[i].id)) {
				return ids[i].distro;
			}
		}
	}
	return PROFILE_DISTRO_UNKNOWN;
}

static unsigned int profile_read_vendor(const struct profile_vaapi_host *host, int node)
{
	char path[64];
	char text[32];

	snprintf(path, sizeof(path), "/sys/class/drm/renderD%d/device/vendor", node);
	if (!host->read_file(host->param, path, text, sizeof(text))) {
		return 0;
	}
	return (unsigned int)strtoul(text, NULL, 16);
}

static unsigned int profile_read_device(const struct profile_vaapi_host *host, int node)
{
	char path[64];
	char text[32];

	snprintf(path, sizeof(path), "/sys/class/drm/renderD%d/device/device", node);
	if (!host->read_file(host->param, path, text, sizeof(text))) {
		return 0;
	}
	return (unsigned int)strtoul(text, NULL, 16);
}

static bool profile_vaapi_driver_installed(const struct profile_vaapi_host *host, const struct profile_vaapi_vendor *v)
{
	char path[128];

	for (const char *const *d = v->drivers; *d; d++) {
		for (const char *const *dir = profile_vaapi_driver_dirs; *dir; dir++) {
			snprintf(path, sizeof(path), "%s/%s_drv_video.so", *dir, *d);
			if (host->exists(host->param, path)) {
				return true;
			}
		}
	}
	return false;
}

void profile_vaapi_diagnose(const struct profile_vaapi_host *host, struct profile_vaapi_diagnosis *out)
{
	static char install_command[96];
	bool found_nvidia = false;
	int node = -1;

	memset(out, 0, sizeof(*out));

	/* The first render device that is not an NVIDIA GPU: libva's DRM
	 * path and obs-ffmpeg try those, NVIDIA has no VA-API encoder. */
	for (int i = 0; i < PROFILE_RENDER_NODE_COUNT; i++) {
		char path[32];
		unsigned int vendor;

		snprintf(path, sizeof(path), "/dev/dri/renderD%d", PROFILE_RENDER_NODE_FIRST + i);
		if (!host->exists(host->param, path)) {
			continue;
		}
		vendor = profile_read_vendor(host, PROFILE_RENDER_NODE_FIRST + i);
		if (vendor == PROFILE_PCI_VENDOR_NVIDIA) {
			if (!found_nvidia) {
				snprintf(out->device, sizeof(out->device), "%s", path);
				out->vendor = vendor;
			}
			found_nvidia = true;
			continue;
		}
		snprintf(out->device, sizeof(out->device), "%s", path);
		out->vendor = vendor;
		node = PROFILE_RENDER_NODE_FIRST + i;
		break;
	}

	if (node < 0) {
		out->cause = found_nvidia ? PROFILE_VAAPI_NVIDIA : PROFILE_VAAPI_NO_RENDER_NODE;
		return;
	}
	if (!host->can_open_rw(host->param, out->device)) {
		out->cause = PROFILE_VAAPI_NO_ACCESS;
		return;
	}

	out->cause = PROFILE_VAAPI_NO_ENCODE;
	for (size_t i = 0; i < sizeof(profile_vaapi_vendors) / sizeof(profile_vaapi_vendors[0]); i++) {
		const struct profile_vaapi_vendor *v = &profile_vaapi_vendors[i];
		enum profile_distro distro;

		if (v->vendor != out->vendor || profile_vaapi_driver_installed(host, v)) {
			continue;
		}
		distro = profile_detect_distro(host);
		out->cause = PROFILE_VAAPI_NO_DRIVER;
		out->package = v->packages[distro];
		if (profile_install_prefix[distro]) {
			snprintf(install_command, sizeof(install_command), "%s%s", profile_install_prefix[distro],
				 out->package);
			out->install_command = install_command;
		}
		break;
	}
}

/* Intel ARK's processor-generation list identifies the families, while the
 * i915 PCI IDs provide these ranges (https://www.intel.com/content/www/us/en/ark.html;
 * https://github.com/torvalds/linux/blob/master/drivers/gpu/drm/i915/intel_device_info.c).
 * Quick Sync begins with Sandy Bridge. oneVPL supports Tiger Lake/Gen11 and
 * newer; older supported generations use Media SDK. Unknown IDs are left
 * unsupported so the dialog never guesses an inapplicable package (#1720). */
enum profile_qsv_family profile_qsv_family_for_device(unsigned int device_id)
{
	static const struct {
		unsigned int first, last;
		enum profile_qsv_family family;
	} ranges[] = {
		{0x0100, 0x012f, PROFILE_QSV_MEDIA_SDK},
		{0x0150, 0x016f, PROFILE_QSV_MEDIA_SDK},
		{0x0400, 0x042f, PROFILE_QSV_MEDIA_SDK},
		{0x0a00, 0x0a2f, PROFILE_QSV_MEDIA_SDK},
		{0x0c00, 0x0c2f, PROFILE_QSV_MEDIA_SDK},
		{0x0d00, 0x0d2f, PROFILE_QSV_MEDIA_SDK},
		{0x1600, 0x163f, PROFILE_QSV_MEDIA_SDK},
		{0x1900, 0x193f, PROFILE_QSV_MEDIA_SDK}, /* Skylake, including HD 530 */
		{0x3100, 0x318f, PROFILE_QSV_MEDIA_SDK},
		{0x5900, 0x593f, PROFILE_QSV_MEDIA_SDK},
		{0x3e00, 0x3eff, PROFILE_QSV_MEDIA_SDK},
		{0x8a00, 0x8aff, PROFILE_QSV_MEDIA_SDK},
		{0x4900, 0x490f, PROFILE_QSV_ONEVPL},
		{0x9a00, 0x9aff, PROFILE_QSV_ONEVPL},
		{0x46a0, 0x46ff, PROFILE_QSV_ONEVPL},
		{0x4c80, 0x4cff, PROFILE_QSV_ONEVPL},
		{0x56a0, 0x56ff, PROFILE_QSV_ONEVPL},
		{0x7d40, 0x7dff, PROFILE_QSV_ONEVPL},
		{0x64a0, 0x64ff, PROFILE_QSV_ONEVPL},
		{0xe200, 0xe2ff, PROFILE_QSV_ONEVPL},
	};
	for (size_t i = 0; i < sizeof(ranges) / sizeof(ranges[0]); i++) {
		if (device_id >= ranges[i].first && device_id <= ranges[i].last) {
			return ranges[i].family;
		}
	}
	return PROFILE_QSV_UNSUPPORTED_FAMILY;
}

static const char *const profile_qsv_runtime_dirs[] = {
	"/run/opengl-driver/lib",
	"/usr/lib/x86_64-linux-gnu",
	"/usr/lib/aarch64-linux-gnu",
	"/usr/lib64",
	"/usr/lib",
	NULL,
};

static bool profile_qsv_runtime_installed(const struct profile_vaapi_host *host, enum profile_qsv_family family)
{
	/* Debian's libmfx1 ships libmfx.so.1 and libmfx-gen1.2 ships
	 * libmfx-gen.so.1.2: https://packages.debian.org/bookworm/amd64/libmfx1/filelist
	 * and https://packages.debian.org/bookworm/amd64/libmfx-gen1.2/filelist.
	 * libvpl.so.2 is the oneVPL dispatcher. */
	static const char *const media_sdk[] = {"libmfx.so.1", NULL};
	static const char *const onevpl[] = {"libvpl.so.2", "libmfx-gen.so.1.2", NULL};
	const char *const *names = family == PROFILE_QSV_MEDIA_SDK ? media_sdk : onevpl;
	char path[128];

	for (; *names; names++) {
		bool found = false;
		for (const char *const *dir = profile_qsv_runtime_dirs; *dir; dir++) {
			snprintf(path, sizeof(path), "%s/%s", *dir, *names);
			if (host->exists(host->param, path)) {
				found = true;
				break;
			}
		}
		if (!found) {
			return false;
		}
	}
	return true;
}

void profile_qsv_diagnose(const struct profile_vaapi_host *host, struct profile_qsv_diagnosis *out)
{
	/* NULL means unverified package naming and deliberately falls back to the
	 * generic dialog. Fedora oneVPL naming is not assumed (#1720, #1863). */
	static const char *const packages[][4] = {
		[PROFILE_QSV_MEDIA_SDK] = {"libmfx1", "libmfx1", "intel-mediasdk", "intel-media-sdk"},
		[PROFILE_QSV_ONEVPL] = {"libmfx-gen1.2 libvpl2", "libmfx-gen1.2 libvpl2", NULL, "vpl-gpu-rt libvpl"},
	};
	static char install_command[128];
	int node = -1;

	memset(out, 0, sizeof(*out));
	for (int i = 0; i < PROFILE_RENDER_NODE_COUNT; i++) {
		char path[32];
		unsigned int vendor;
		snprintf(path, sizeof(path), "/dev/dri/renderD%d", PROFILE_RENDER_NODE_FIRST + i);
		if (!host->exists(host->param, path)) {
			continue;
		}
		vendor = profile_read_vendor(host, PROFILE_RENDER_NODE_FIRST + i);
		if (vendor != PROFILE_PCI_VENDOR_INTEL) {
			continue;
		}
		snprintf(out->device, sizeof(out->device), "%s", path);
		out->vendor = vendor;
		out->device_id = profile_read_device(host, PROFILE_RENDER_NODE_FIRST + i);
		node = PROFILE_RENDER_NODE_FIRST + i;
		break;
	}
	if (node < 0) {
		out->cause = PROFILE_QSV_NO_INTEL_GPU;
		return;
	}
	out->family = profile_qsv_family_for_device(out->device_id);
	if (out->family == PROFILE_QSV_UNSUPPORTED_FAMILY) {
		out->cause = PROFILE_QSV_UNSUPPORTED;
		return;
	}
	if (!profile_qsv_runtime_installed(host, out->family)) {
		enum profile_distro distro = profile_detect_distro(host);
		out->cause = PROFILE_QSV_RUNTIME_MISSING;
		out->package = packages[out->family][distro];
		if (out->package && profile_install_prefix[distro]) {
			snprintf(install_command, sizeof(install_command), "%s%s", profile_install_prefix[distro],
				 out->package);
			out->install_command = install_command;
		}
		return;
	}
	if (!host->can_open_rw(host->param, out->device)) {
		out->cause = PROFILE_QSV_NO_ACCESS;
		return;
	}
	out->cause = PROFILE_QSV_OPERATIONAL;
}

#undef PROFILE_PCI_VENDOR_INTEL
#undef PROFILE_PCI_VENDOR_AMD
#undef PROFILE_PCI_VENDOR_NVIDIA
#undef PROFILE_RENDER_NODE_FIRST
#undef PROFILE_RENDER_NODE_COUNT
