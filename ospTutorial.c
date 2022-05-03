// Copyright 2009-2021 Intel Corporation
// SPDX-License-Identifier: Apache-2.0

/* This is a small example tutorial how to use OSPRay in an application.
 *
 * On Linux build it in the build_directory with
 *   gcc -std=c99 ../apps/ospTutorial/ospTutorial.c \
 *       -I ../ospray/include -L . -lospray -Wl,-rpath,. -o ospTutorial
 * On Windows build it in the build_directory\$Configuration with
 *   cl ..\..\apps\ospTutorial\ospTutorial.c -I ..\..\ospray\include ^
 *       -I ..\.. ospray.lib
 */

 #include <stdlib.h>

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#ifdef _WIN32
#include <conio.h>
#include <malloc.h>
#include <windows.h>
#else
#include <alloca.h>
#endif
#include "ospray/ospray_util.h"

// helper function to write the rendered image as PPM file
void writePPM(
    const char *fileName, int size_x, int size_y, const uint32_t *pixel)
{
  FILE *file = fopen(fileName, "wb");
  if (!file) {
    fprintf(stderr, "fopen('%s', 'wb') failed: %d", fileName, errno);
    return;
  }
  fprintf(file, "P6\n%i %i\n255\n", size_x, size_y);
  unsigned char *out = (unsigned char *)alloca(3 * size_x);
  for (int y = 0; y < size_y; y++) {
    const unsigned char *in =
        (const unsigned char *)&pixel[(size_y - 1 - y) * size_x];
    for (int x = 0; x < size_x; x++) {
      out[3 * x + 0] = in[4 * x + 0];
      out[3 * x + 1] = in[4 * x + 1];
      out[3 * x + 2] = in[4 * x + 2];
    }
    fwrite(out, 3 * size_x, sizeof(char), file);
  }
  fprintf(file, "\n");
  fclose(file);
}

int main(int argc, const char **argv)
{
  // image size
  int imgSize_x = 700; // width
  int imgSize_y = 700; // height

  // camera
  float cam_pos[] = {849.85864f, 812.4856f, 883.1134f};
  float cam_up[] = {0.f, 1.f, 0.f};
  float cam_view[] = {-0.57735026f, -0.57735026f, -0.57735026f};

  // triangle mesh data
  float vertex[] = {-1.0f,
      -1.0f,
      3.0f,
      -1.0f,
      1.0f,
      3.0f,
      1.0f,
      -1.0f,
      3.0f,
      0.1f,
      0.1f,
      0.3f};
  float color[] = {0.9f,
      0.5f,
      0.5f,
      1.0f,
      0.8f,
      0.8f,
      0.8f,
      1.0f,
      0.8f,
      0.8f,
      0.8f,
      1.0f,
      0.5f,
      0.9f,
      0.5f,
      1.0f};
  int32_t index[] = {0, 1, 2, 1, 2, 3};

#ifdef _WIN32
  int waitForKey = 0;
  CONSOLE_SCREEN_BUFFER_INFO csbi;
  if (GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &csbi)) {
    // detect standalone console: cursor at (0,0)?
    waitForKey = csbi.dwCursorPosition.X == 0 && csbi.dwCursorPosition.Y == 0;
  }
#endif

  printf("initialize OSPRay...");

  // initialize OSPRay; OSPRay parses (and removes) its commandline parameters,
  // e.g. "--osp:debug"
  OSPError init_error = ospInit(&argc, argv);
  if (init_error != OSP_NO_ERROR)
    return init_error;

  printf("done\n");
  printf("setting up camera...");

  // create and setup camera
  OSPCamera camera = ospNewCamera("perspective");
  ospSetFloat(camera, "aspect", imgSize_x / (float)imgSize_y);
  ospSetParam(camera, "position", OSP_VEC3F, cam_pos);
  ospSetParam(camera, "direction", OSP_VEC3F, cam_view);
  ospSetParam(camera, "up", OSP_VEC3F, cam_up);
  ospCommit(camera); // commit each object to indicate modifications are done

  printf("done\n");
  printf("setting up scene...");

  // create and setup model and mesh
  OSPGeometry mesh = ospNewGeometry("mesh");

  OSPData data = ospNewSharedData1D(vertex, OSP_VEC3F, 4);
  // alternatively with an OSPRay managed OSPData
  // OSPData managed = ospNewData1D(OSP_VEC3F, 4);
  // ospCopyData1D(data, managed, 0);

  ospCommit(data);
  ospSetObject(mesh, "vertex.position", data);
  ospRelease(data); // we are done using this handle

  data = ospNewSharedData1D(color, OSP_VEC4F, 4);
  ospCommit(data);
  ospSetObject(mesh, "vertex.color", data);
  ospRelease(data);

  data = ospNewSharedData1D(index, OSP_VEC3UI, 2);
  ospCommit(data);
  ospSetObject(mesh, "index", data);
  ospRelease(data);

  ospCommit(mesh);

  OSPMaterial mat =
    ospNewMaterial("", "obj"); // first argument no longer matters
  ospCommit(mat);

  // put the mesh into a model
  OSPGeometricModel model = ospNewGeometricModel(mesh);
  ospSetObject(model, "material", mat);
  ospCommit(model);
  ospRelease(mesh);
  ospRelease(mat);

  // MY

  FILE *f = fopen("/mnt/vdrive/projects/raycaster/volumes/800shapes_lin.vol", "rb");
  fseek(f, 0, SEEK_END);
  long fsize = ftell(f);
  fseek(f, 0, SEEK_SET);  /* same as rewind(f); */

  char *buffer = malloc(fsize + 1);
  fread(buffer, fsize, 1, f);
  fclose(f);

  printf("size-- %ld\n", fsize);

  OSPVolume vol = ospNewVolume("structuredRegular");
  OSPData vol_data = ospNewSharedData(buffer + 26,
        OSP_UCHAR,
        800,
        0,
        800,
        0,
        800,
        0);
  ospCommit(vol_data);
  ospSetObject(vol, "data", vol_data);
  ospRelease(vol_data);
  ospCommit(vol);

  printf("Data done\n");
  
  OSPTransferFunction tf = ospNewTransferFunction("piecewiseLinear");

//   pub fn shapes_tf(sample: f32) -> RGBA {
//     // relevant data between 90 and 110
//     if sample > 85.0 && sample <= 95.0 {
//         RGBA::new(255.0, 30.0, 60.0, 0.02)
//     } else if sample > 95.0 && sample <= 100.0 {
//         RGBA::new(10.0, 60.0, 180.0, 0.3)
//     } else if sample > 100.0 && sample < 115.0 {
//         RGBA::new(90.0, 210.0, 20.0, 0.6)
//     } else {
//         color::zero()
//     }
// }

  float opacity[52] = {0};
  // opacity[17] = 0.02f;
  // opacity[18] = 0.3f;
  // opacity[19] = 0.3f;
  // opacity[20] = 0.3f;
  // opacity[21] = 0.6f;
  // opacity[22] = 0.6f;
  // opacity[23] = 0.6f;

  opacity[17] = 1.0f;
  opacity[18] = 1.0f;
  opacity[19] = 1.0f;
  opacity[20] = 1.0f;
  opacity[21] = 1.0f;
  opacity[22] = 1.0f;
  opacity[23] = 1.0f;


  int elements = sizeof(opacity)/sizeof(opacity[0]);
  printf("Opacity array of len %d\n", elements);
  data = ospNewSharedData1D(opacity, OSP_FLOAT, elements);
  ospCommit(data);
  ospSetObject(tf, "opacity", data);
  ospRelease(data);

  float tf_color[52*3] = {0};
  tf_color[17*3] = 255.0f/255.0f;
  tf_color[17*3 + 1] = 30.0f/255.0f;
  tf_color[17*3 + 2] = 60.0f/255.0f;

  tf_color[18*3] = 255.0f/255.0f;
  tf_color[18*3 + 1] = 30.0f/255.0f;
  tf_color[18*3 + 2] = 60.0f/255.0f;

  tf_color[19*3] = 10.0f/255.0f;
  tf_color[19*3 + 1] = 60.0f/255.0f;
  tf_color[19*3 + 2] = 180.0f/255.0f;

  tf_color[20*3] = 10.0f/255.0f;
  tf_color[20*3 + 1] = 60.0f/255.0f;
  tf_color[20*3 + 2] = 180.0f/255.0f;

  tf_color[21*3] = 10.0f/255.0f;
  tf_color[21*3 + 1] = 60.0f/255.0f;
  tf_color[21*3 + 2] = 180.0f/255.0f;

  tf_color[22*3] = 90.0f/255.0f;
  tf_color[22*3 + 1] = 210.0f/255.0f;
  tf_color[22*3 + 2] = 20.0f/255.0f;

  tf_color[23*3] = 90.0f/255.0f; 
  tf_color[23*3 + 1] = 210.0f/255.0f;
  tf_color[23*3 + 2] = 20.0f/255.0f;

  //float tf_color_r[3] = {1.0f};

  int elements_col = sizeof(tf_color)/sizeof(tf_color[0])/3;
  printf("Colors n %d\n", elements_col);
  data = ospNewSharedData1D(tf_color, OSP_VEC3F, elements_col);
  ospCommit(data);
  ospSetObject(tf, "color", data);
  ospRelease(data);

  ospCommit(tf);

  printf("TF done\n");

  OSPVolumetricModel vol_model = ospNewVolumetricModel(vol);
  ospSetObject(vol_model, "transferFunction", tf);
  ospCommit(vol_model);

  // put the model into a group (collection of models)
  OSPGroup group = ospNewGroup();
  ospSetObjectAsData(group, "volume", OSP_VOLUMETRIC_MODEL, vol_model);
  ospCommit(group);

  ospRelease(model);

  printf("Model done\n");

  // put the group into an instance (give the group a world transform)
  OSPInstance instance = ospNewInstance(group);
  ospCommit(instance);
  ospRelease(group);

  // put the instance in the world
  OSPWorld world = ospNewWorld();
  ospSetObjectAsData(world, "instance", OSP_INSTANCE, instance);
  ospRelease(instance);

  // create and setup light for Ambient Occlusion
  OSPLight light = ospNewLight("ambient");
  ospCommit(light);
  ospSetObjectAsData(world, "light", OSP_LIGHT, light);
  ospRelease(light);

  ospCommit(world);

  printf("done\n");

  // print out world bounds
  OSPBounds worldBounds = ospGetBounds(world);
  printf("world bounds: ({%f, %f, %f}, {%f, %f, %f}\n\n",
      worldBounds.lower[0],
      worldBounds.lower[1],
      worldBounds.lower[2],
      worldBounds.upper[0],
      worldBounds.upper[1],
      worldBounds.upper[2]);

  printf("setting up renderer...");

  // create renderer
  OSPRenderer renderer =
      ospNewRenderer("ao"); // choose path tracing renderer

  // complete setup of renderer
  ospSetFloat(renderer, "volumeSamplingRate", 50.0f); // white, transparent
  ospCommit(renderer);

  // create and setup framebuffer
  OSPFrameBuffer framebuffer = ospNewFrameBuffer(imgSize_x,
      imgSize_y,
      OSP_FB_SRGBA,
      OSP_FB_COLOR | /*OSP_FB_DEPTH |*/ OSP_FB_ACCUM);
  ospResetAccumulation(framebuffer);

  printf("rendering initial frame to firstFrame.ppm...");

  // render one frame
  ospRenderFrameBlocking(framebuffer, renderer, camera, world);

  // access framebuffer and write its content as PPM file
  const uint32_t *fb = (uint32_t *)ospMapFrameBuffer(framebuffer, OSP_FB_COLOR);
  writePPM("firstFrame.ppm", imgSize_x, imgSize_y, fb);
  ospUnmapFrameBuffer(fb, framebuffer);

  printf("done\n");
  printf("rendering 10 accumulated frames to accumulatedFrame.ppm...");

  // render 10 more frames, which are accumulated to result in a better
  // converged image
  for (int frames = 0; frames < 10; frames++) {

    ospRenderFrameBlocking(framebuffer, renderer, camera, world);
    printf("done %d\n", frames);
  }

  fb = (uint32_t *)ospMapFrameBuffer(framebuffer, OSP_FB_COLOR);
  writePPM("accumulatedFrame.ppm", imgSize_x, imgSize_y, fb);
  ospUnmapFrameBuffer(fb, framebuffer);

  printf("done\n\n");

  OSPPickResult p;
  ospPick(&p, framebuffer, renderer, camera, world, 0.5f, 0.5f);

  printf("ospPick() center of screen --> [inst: %p, model: %p, prim: %u]\n",
      p.instance,
      p.model,
      p.primID);

  printf("cleaning up objects...");

  // cleanup pick handles (because p.hasHit was 'true')
  ospRelease(p.instance);
  ospRelease(p.model);

  // final cleanups
  ospRelease(renderer);
  ospRelease(camera);
  ospRelease(framebuffer);
  ospRelease(world);

  free(buffer);

  printf("done\n");

  ospShutdown();

#ifdef _WIN32
  if (waitForKey) {
    printf("\n\tpress any key to exit");
    _getch();
  }
#endif

  return 0;
}
