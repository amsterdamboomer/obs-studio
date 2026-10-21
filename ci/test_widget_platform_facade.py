#!/usr/bin/env -S uv run --script
"""Widget handle/event/ownership policy before and after extraction (#875)."""
from pathlib import Path
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parent))
import platform_ifdef_ratchet as ratchet
from test_settings_platform_facade import function
ROOT=Path(__file__).resolve().parent.parent
WIDGETS=ROOT/'frontend/widgets'
UTILITY=ROOT/'frontend/utility'
OPS=('void ApplyPlatformPreviewTextOutline(', 'void SetPlatformWidgetIcon(',
     'void ConfigurePlatformProjectorWindow(', 'bool FillPlatformGraphicsWindow(',
     'bool PlatformResetsHiddenDisplay(', 'void HandlePlatformDisplayNativeEvent(')
FILES=('OBSBasicPreview','OBSBasicStats','OBSProjector','OBSQTDisplay')


def text(name):
    return (WIDGETS/(name+'.cpp.hpp')).read_text()


def original_block(name,start):
    return re.search(start+r'.*?#endif',text(name),re.S)[0]


def bodies(host):
    native=(UTILITY/f'platform-{host}.hpp').read_text()
    if OPS[0] in native:
        return '\n'.join(function(native,op) for op in OPS)
    outline=original_block('OBSBasicPreview',r'#ifdef _WIN32\n\s*obs_data_set_int\(settings, "outline_color"')
    outline=re.sub(r'^\s*const char \*text_source_id.*$', '',outline,flags=re.M)
    icon=original_block('OBSBasicStats',r'#ifndef __APPLE__').replace('setWindowIcon(', 'widget->setWindowIcon(')
    projector=original_block('OBSProjector',r'#if defined\(__linux__\)').replace('setAttribute(', 'widget->setAttribute(')
    window=function(text('OBSQTDisplay'),'static bool QTToGSWindow(').replace('static bool QTToGSWindow(', 'bool FillPlatformGraphicsWindow(')
    reset=original_block('OBSQTDisplay',r'#if !defined\(_WIN32\)').replace('display = nullptr;', 'return true;')
    event=function(text('OBSQTDisplay'),'bool OBSQTDisplay::nativeEvent(')
    event=event[event.index('{')+1:event.rindex('}')].replace('OnDisplayChange();','display->OnDisplayChange();').replace('return false;','')
    return ('void ApplyPlatformPreviewTextOutline(obs_data_t *settings){\n'+outline+'\n}\n'
            'void SetPlatformWidgetIcon(QWidget *widget){\n'+icon+'\n}\n'
            'void ConfigurePlatformProjectorWindow(QWidget *widget){\n'+projector+'\n}\n'+window+'\n'
            'bool PlatformResetsHiddenDisplay(){\n'+reset+'\nreturn false;}\n'
            'void HandlePlatformDisplayNativeEvent(OBSQTDisplay *display,void *message){\n'+event+'\n}\n')


def preview_settings():
    source=text('OBSBasicPreview')
    start=source.index('OBSDataAutoRelease font = obs_data_create();',source.index('obs_source_t *OBSBasicPreview::')) if 'obs_source_t *OBSBasicPreview::' in source else source.index('OBSDataAutoRelease font = obs_data_create();')
    start=source.index('\n',start)+1
    end=source.index('DStr name;',start)
    return 'const char *PreviewSettings(obs_data_t *font,obs_data_t *settings,double pixelRatio){'+source[start:end]+'return text_source_id;}\n'


def visible_callback():
    callback=function(text('OBSQTDisplay'),'auto windowVisible = [this](bool visible)')
    return 'std::function<void(bool)> OBSQTDisplay::VisibleCallback(){'+callback+';return windowVisible;}\n'


COMMON=r'''
#include <cassert>
#include <cstdint>
#include <string>
#include <map>
#include <functional>
#include <vector>
'''
STUBS=r'''
#define UNUSED_PARAMETER(x) (void)x
#define __bridge
using id=void*;using HWND=void*;using WId=uintptr_t;
namespace Qt{constexpr int WA_PaintOnScreen=1;}
struct QIcon{std::string name;QIcon(const char *value):name(value){}static QIcon fromTheme(const char *name,const QIcon &fallback){assert(std::string(name)=="obs"&&fallback.name==":/res/images/obs.png");return QIcon("theme:obs");}};
struct QWidget{int attributes=0,icons=0;bool paint=true;void setAttribute(int key,bool value=true){assert(key==1);++attributes;paint=value;}void setWindowIcon(const QIcon &icon){assert(icon.name=="theme:obs");++icons;}};
struct QWindow{WId handle=0x12345;int reads=0;WId winId(){++reads;return handle;}};
struct gs_window{void *hwnd=(void*)1,*view=(void*)2;uint32_t id=3;void *display=(void*)4;};
enum obs_nix_platform_type{OBS_NIX_PLATFORM_INVALID,OBS_NIX_PLATFORM_X11_EGL,OBS_NIX_PLATFORM_WAYLAND};
obs_nix_platform_type backend=OBS_NIX_PLATFORM_X11_EGL;int displayReads=0;
obs_nix_platform_type obs_get_nix_platform(){return backend;}void *obs_get_nix_platform_display(){++displayReads;return (void*)0x9876;}
struct obs_data_t{std::map<std::string,long long> ints;std::map<std::string,std::string> strings;bool outline=false;obs_data_t *font=nullptr;};
void obs_data_set_int(obs_data_t *data,const char *key,long long value){data->ints[key]=value;}
void obs_data_set_string(obs_data_t *data,const char *key,const char *value){data->strings[key]=value;}
void obs_data_set_bool(obs_data_t *data,const char *key,bool value){assert(std::string(key)=="outline");data->outline=value;}
void obs_data_set_obj(obs_data_t *data,const char *key,obs_data_t *value){assert(std::string(key)=="font");data->font=value;}
struct QByteArray{};using qintptr=long long;struct MSG{int message;};constexpr int WM_MOVE=1,WM_DISPLAYCHANGE=2;
int destroyed=0,resized=0;
struct obs_display_t{};void obs_display_destroy(obs_display_t *display){if(display){++destroyed;delete display;}}
void obs_display_resize(obs_display_t *display,int w,int h){assert(display&&w==640&&h==480);++resized;}
'''


class WidgetPlatformFacadeTest(unittest.TestCase):
    def test_shared_boundary(self):
        for name in FILES:
            with self.subTest(file=name):
                self.assertEqual(ratchet.count_platform_conditionals(text(name)),[])
        p='frontend/build/frontend-widgets-obsqtdisplay.pch.hpp'
        self.assertEqual(ratchet.findings(p,(ROOT/p).read_text()),[])

    def test_all_hosts_declare_and_define_operations(self):
        for op in OPS:
            self.assertIn(op,(UTILITY/'platform.hpp').read_text())
            for host in ('windows.cpp','osx.mm','x11.cpp'):
                self.assertIn(op,(UTILITY/f'platform-{host}.hpp').read_text())

    def test_original_policy_and_borrowed_handles(self):
        compiler=shutil.which('c++')
        if not compiler:
            self.skipTest('Native compiler unavailable')
        owner=function((ROOT/'libobs/obs.hpp').read_text(),'template<typename T, void destroy(T)> class OBSPtr')+';\n'
        classes=r'''
using OBSDisplay=OBSPtr<obs_display_t*,obs_display_destroy>;
struct OBSQTDisplay:QWidget{OBSDisplay display;int created=0,changes=0;void CreateDisplay(){display=new obs_display_t;++created;}void OnDisplayChange(){++changes;}bool nativeEvent(const QByteArray&,void*,qintptr*);std::function<void(bool)> VisibleCallback();};
struct QSize{int width()const{return 640;}int height()const{return 480;}};QSize GetPixelSize(QWidget*){return {};}
'''
        variants=[('windows.cpp','_WIN32',False,False),('osx.mm','__APPLE__',False,False),('x11.cpp','__linux__',False,True),('x11.cpp','__linux__',True,True),('x11.cpp','__FreeBSD__',True,True),('x11.cpp','__DragonFly__',False,True),('x11.cpp','__OpenBSD__',False,False)]
        for host,macro,wayland,projector in variants:
            win=host=='windows.cpp';mac=host=='osx.mm';nix=not win and not mac
            with self.subTest(host=host,macro=macro,wayland=wayland),tempfile.TemporaryDirectory(prefix='obs-widget-policy-') as tmp:
                native=(UTILITY/f'platform-{host}.hpp').read_text()
                extras='\n'.join(function(native,s) for s in ('const char *GetPlatformPropertiesFontFace(','const char *GetPlatformPropertiesTextSource('))
                checks=f'''
int main(){{
 QWidget widget;SetPlatformWidgetIcon(&widget);assert(widget.icons=={0 if mac else 1});
 ConfigurePlatformProjectorWindow(&widget);assert(widget.attributes=={int(projector)}&&widget.paint=={str(not projector).lower()});
 for(double ratio:{{1.0,1.5,2.0}}){{obs_data_t font,settings;settings.ints["outline_color"]=17;settings.ints["outline_size"]=9;
 assert(std::string(PreviewSettings(&font,&settings,ratio))=="{'text_gdiplus' if win else 'text_ft2_source'}");
 assert(font.strings["face"]=="{'Arial' if win else 'Helvetica' if mac else 'Monospace'}"&&font.ints["flags"]==1&&font.ints["size"]==16*ratio);
 assert(settings.outline&&settings.font==&font&&settings.ints["outline_color"]=={0 if win else 17}&&settings.ints["outline_size"]=={3 if win else 9});}}
 for(auto selected:{{OBS_NIX_PLATFORM_INVALID,OBS_NIX_PLATFORM_X11_EGL,OBS_NIX_PLATFORM_WAYLAND}})for(WId handle:{{WId(0),WId(0x12345)}}){{
  backend=selected;QWindow window;window.handle=handle;gs_window gs;displayReads=0;
  bool result=FillPlatformGraphicsWindow(&window,gs);
  bool supported={str(not nix).lower()}||selected==OBS_NIX_PLATFORM_X11_EGL||({str(wayland).lower()}&&selected==OBS_NIX_PLATFORM_WAYLAND);
  bool expected=supported&&(!{str(nix and wayland).lower()}||selected!=OBS_NIX_PLATFORM_WAYLAND||handle!=0);
  assert(result==expected);assert(window.reads==int(supported));
  assert(gs.hwnd==({'(void*)handle' if win else '(void*)1'}));assert(gs.view==({'(void*)handle' if mac else '(void*)2'}));
  assert(gs.id==({str(nix).lower()}&&selected==OBS_NIX_PLATFORM_X11_EGL?handle:3));
  assert(gs.display==({str(nix).lower()}&&selected==OBS_NIX_PLATFORM_X11_EGL?(void*)0x9876:({str(nix and wayland).lower()}&&selected==OBS_NIX_PLATFORM_WAYLAND?(void*)handle:(void*)4)));
  assert(displayReads==int({str(nix).lower()}&&selected==OBS_NIX_PLATFORM_X11_EGL));
 }}
 {{OBSQTDisplay display;qintptr result=99;for(int tag:{{WM_MOVE,WM_DISPLAYCHANGE,999}}){{MSG msg{{tag}};assert(!display.nativeEvent({{}},&msg,&result));assert(result==99);}}assert(display.changes=={int(win)});}}
 destroyed=resized=0;
 {{OBSQTDisplay display;display.CreateDisplay();auto visible=display.VisibleCallback();visible(false);assert(destroyed=={int(nix)});
 visible(true);assert(display.created=={2 if nix else 1}&&resized=={0 if nix else 1});visible(true);assert(resized=={1 if nix else 2});
 visible(false);assert(destroyed=={2 if nix else 0});}}
 assert(destroyed=={2 if nix else 1});
}}
'''
                source=Path(tmp)/'policy.cpp';binary=Path(tmp)/'policy'
                source.write_text(COMMON+'\n#undef __linux__\n#define '+macro+' 1\n'+('#define ENABLE_WAYLAND 1\n' if wayland else '')+STUBS+owner+classes+bodies(host)+extras+preview_settings()+visible_callback()+function(text('OBSQTDisplay'),'bool OBSQTDisplay::nativeEvent(')+checks)
                result=subprocess.run([compiler,'-std=c++17',str(source),'-o',str(binary)],capture_output=True,text=True)
                self.assertEqual(result.returncode,0,result.stderr)
                subprocess.run([str(binary)],check=True)

    def test_objc_arc_view_is_borrowed(self):
        native=(UTILITY/'platform-osx.mm.hpp').read_text()
        if OPS[3] not in native:
            self.skipTest('ARC check applies after extraction into the ARC group')
        compiler=shutil.which('clang++')
        if not compiler:
            self.skipTest('Clang unavailable; native macOS build remains required')
        code='struct QWindow{unsigned long winId();};struct gs_window{__unsafe_unretained id view;};\n'+function(native,OPS[3])
        result=subprocess.run([compiler,'-x','objective-c++','-std=c++17','-fobjc-arc','-fobjc-runtime=macosx-11.0','-O1','-S','-emit-llvm','-o','-','-'],input=code,capture_output=True,text=True)
        self.assertEqual(result.returncode,0,result.stderr)
        self.assertNotRegex(result.stdout,r'@(?:llvm\.)?objc[._](retain|release|storeStrong)')


if __name__=='__main__':
    unittest.main()
