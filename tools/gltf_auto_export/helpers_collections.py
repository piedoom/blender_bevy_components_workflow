import bpy
from .helpers import traverse_tree

# returns the list of the collections in use for a given scene
def get_used_collections(scene): 
    root_collection = scene.collection 

    scene_objects = [o for o in root_collection.objects]
    collection_names = set()
    used_collections = []
    for object in scene_objects:
        #print("object ", object)
        if object.instance_type == 'COLLECTION':
            #print("THIS OBJECT IS A COLLECTION")
            # print("instance_type" ,object.instance_type)
            collection_name = object.instance_collection.name
            #print("instance collection", object.instance_collection.name)
            #object.instance_collection.users_scene
            # del object['blueprint']
            # object['BlueprintName'] = '"'+collection_name+'"'
            if not collection_name in collection_names: 
                collection_names.add(collection_name)
                used_collections.append(object.instance_collection)

    #print("scene objects", scene_objects) 
    return (collection_names, used_collections)

# gets all collections that should ALWAYS be exported to their respective gltf files, even if they are not used in the main scene/level
def get_marked_collections(scene):
    # print("checking library for marked collections")
    root_collection = scene.collection
    marked_collections = []
    collection_names = []
    for collection in traverse_tree(root_collection):
        if 'AutoExport' in collection and collection['AutoExport'] == True:
            marked_collections.append(collection)
            collection_names.append(collection.name)
    return (collection_names, marked_collections)

# get exportable collections from lists of mains scenes and lists of library scenes
def get_exportable_collections(main_scenes, library_scenes): 
    all_collections = []
    for main_scene in main_scenes:
        (collection_names, _) = get_used_collections(main_scene)
        all_collections = all_collections + list(collection_names)
    for library_scene in library_scenes:
        marked_collections = get_marked_collections(library_scene)
        all_collections = all_collections + marked_collections[0]
    return all_collections

def get_collections_per_scene(collection_names, library_scenes): 
    collections_per_scene = {}
    for scene in library_scenes:
        root_collection = scene.collection
        for cur_collection in traverse_tree(root_collection):
            if cur_collection.name in collection_names:
                if not scene.name in collections_per_scene:
                    collections_per_scene[scene.name] = []
                collections_per_scene[scene.name].append(cur_collection.name)
                
    return collections_per_scene


def get_collection_hierarchy(root_col, levels=1):
    """Read hierarchy of the collections in the scene"""
    level_lookup = {}
    def recurse(root_col, parent, depth):
        if depth > levels: 
            return
        if isinstance(parent,  bpy.types.Collection):
            level_lookup.setdefault(parent, []).append(root_col)
        for child in root_col.children:
            recurse(child, root_col,  depth + 1)
    recurse(root_col, root_col.children, 0)
    return level_lookup

# the active collection is a View Layer concept, so you actually have to find the active LayerCollection
# which must be done recursively
def find_layer_collection_recursive(find, col):
    # print("root collection", col)
    for c in col.children:
        # print("child collection", c)
        if c.collection == find:
            return c
    return None


#Recursivly transverse layer_collection for a particular name
def recurLayerCollection(layerColl, collName):
    found = None
    if (layerColl.name == collName):
        return layerColl
    for layer in layerColl.children:
        found = recurLayerCollection(layer, collName)
        if found:
            return found

