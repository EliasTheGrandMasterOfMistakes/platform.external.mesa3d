/*
 * Copyright © 2016 Red Hat.
 * Copyright © 2016 Bas Nieuwenhuizen
 *
 * based in part on anv driver which is:
 * Copyright © 2015 Intel Corporation
 *
 * SPDX-License-Identifier: MIT
 */

#ifndef RADV_VIDEO_H
#define RADV_VIDEO_H

#include "vk_video.h"

#define VL_MACROBLOCK_WIDTH  16
#define VL_MACROBLOCK_HEIGHT 16

struct radv_physical_device;

struct radv_vid_mem {
   struct radv_device_memory *mem;
   VkDeviceSize offset;
   VkDeviceSize size;
};

struct radv_video_session {
   struct vk_video_session vk;

   uint32_t stream_handle;
   unsigned stream_type;
   bool interlaced;
   enum { DPB_MAX_RES = 0, DPB_DYNAMIC_TIER_1, DPB_DYNAMIC_TIER_2 } dpb_type;
   unsigned db_alignment;

   struct radv_vid_mem sessionctx;
   struct radv_vid_mem ctx;

   unsigned dbg_frame_cnt;
};

VK_DEFINE_NONDISP_HANDLE_CASTS(radv_video_session, vk.base, VkVideoSessionKHR, VK_OBJECT_TYPE_VIDEO_SESSION_KHR)

struct radv_video_session_params {
   struct vk_video_session_parameters vk;
};

VK_DEFINE_NONDISP_HANDLE_CASTS(radv_video_session_params, vk.base, VkVideoSessionParametersKHR,
                               VK_OBJECT_TYPE_VIDEO_SESSION_PARAMETERS_KHR)

void radv_init_physical_device_decoder(struct radv_physical_device *pdev);

void radv_video_get_profile_alignments(struct radv_physical_device *pdev, const VkVideoProfileListInfoKHR *profile_list,
                                       uint32_t *width_align_out, uint32_t *height_align_out);

#endif /* RADV_VIDEO_H */