# Result


```bash
Validation layer available: true
VK_EXT_debug_utils: true
VK_KHR_portability_enumeration: true
VK_KHR_get_physical_device_properties2: true
Enabling Vulkan portability enumeration
Enabling validation layers
Instance created!
Found 1 physical device(s)
----------------------------------------
Selected GPU: Apple M4 Max
Vulkan API: 1.0.357
Driver version: 10402
Graphics queue family: 0
----------------------------------------
Validation Error: [ VUID-VkDeviceCreateInfo-pProperties-04451 ] | MessageID = 0x3a3b6ca0
vkCreateDevice(): VK_KHR_portability_subset must be enabled because physical device VkPhysicalDevice 0x7baf4a9320 supports it.
The Vulkan spec states: If the VK_KHR_portability_subset extension is included in pProperties of vkEnumerateDeviceExtensionProperties, ppEnabledExtensionNames must include "VK_KHR_portability_subset" (https://vulkan.lunarg.com/doc/view/1.4.357.1/mac/antora/spec/latest/chapters/devsandqueues.html#VUID-VkDeviceCreateInfo-pProperties-04451)
Objects: 1
    [0] VkPhysicalDevice 0x7baf4a9320

Logical device created!
Graphics queue created!
Vulkan initialization successful!

```

