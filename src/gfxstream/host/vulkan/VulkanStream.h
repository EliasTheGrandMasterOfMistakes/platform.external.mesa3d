// Copyright (C) 2018 The Android Open Source Project
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#pragma once

#include <inttypes.h>

#include <memory>
#include <vector>

#include "VulkanHandleMapping.h"
#include "aemu/base/BumpPool.h"
#include "aemu/base/files/Stream.h"
#include "aemu/base/files/StreamSerializing.h"
#include "goldfish_vk_private_defs.h"

#define E(fmt, ...) fprintf(stderr, fmt "\n", ##__VA_ARGS__)

namespace android {
namespace base {
class BumpPool;
}  // namespace base
}  // namespace android

namespace gfxstream {
class IOStream;
}  // namespace gfxstream

namespace gfxstream {
namespace vk {

class VulkanStream : public android::base::Stream {
   public:
    VulkanStream(IOStream* stream);
    ~VulkanStream();

    void setStream(IOStream* stream);

    // Returns whether the connection is valid.
    bool valid();

    // General allocation function
    void alloc(void** ptrAddr, size_t bytes);

    // Utility functions to load strings or
    // string arrays in place with allocation.
    void loadStringInPlace(char** forOutput);
    void loadStringArrayInPlace(char*** forOutput);

    // When we load a string and are using a reserved pointer.
    void loadStringInPlaceWithStreamPtr(char** forOutput, uint8_t** streamPtr);
    void loadStringArrayInPlaceWithStreamPtr(char*** forOutput, uint8_t** streamPtr);

    virtual ssize_t read(void* buffer, size_t size);
    virtual ssize_t write(const void* buffer, size_t size);

    void commitWrite();

    // Frees everything that got alloc'ed.
    void clearPool();

    void setHandleMapping(VulkanHandleMapping* mapping);
    void unsetHandleMapping();
    VulkanHandleMapping* handleMapping() const;

    uint32_t getFeatureBits() const;

    android::base::BumpPool* pool();

   private:
    size_t remainingWriteBufferSize() const;
    ssize_t bufferedWrite(const void* buffer, size_t size);
    android::base::BumpPool mPool;
    size_t mWritePos = 0;
    std::vector<uint8_t> mWriteBuffer;
    IOStream* mStream = nullptr;
    DefaultHandleMapping mDefaultHandleMapping;
    VulkanHandleMapping* mCurrentHandleMapping;
    uint32_t mFeatureBits = 0;
};

class VulkanMemReadingStream : public VulkanStream {
   public:
    VulkanMemReadingStream(uint8_t* start);
    ~VulkanMemReadingStream();

    void setBuf(uint8_t* buf);
    uint8_t* getBuf();
    void setReadPos(uintptr_t pos);

    ssize_t read(void* buffer, size_t size) override;
    ssize_t write(const void* buffer, size_t size) override;

    uint8_t* beginTrace();
    size_t endTrace();

   private:
    void resetTrace();

    uint8_t* mStart;
    uint8_t* mTraceStart;
    uintptr_t mReadPos = 0;
};

}  // namespace vk
}  // namespace gfxstream